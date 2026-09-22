//! What crosses the boundary an `extern` declaration is, and what a Java name is.
//!
//! `docs/specs/interop.md` states both. A signature is the whole of the boundary: every type in
//! it compiles to exactly the JVM type the member's own descriptor holds, so the types that may
//! be written are the ones that have one. What a declaration cannot say is not reachable, and
//! the answer to that is another declaration rather than a rule that widens one of these.

use std::collections::HashMap;

use lumen_ast::{Called, ExternDeclaration, ExternParameter, JavaName, Reaches, Span, Takes};

use crate::error::TypeError;
use crate::error::reaching_java::ReachingJava;
use crate::types::{LIST, OPTION, RESULT, Type};

/// How a method of each extern type is called, by the Lumen name an `extern type` gives it.
pub(crate) type Foreign = HashMap<String, Called>;

/// Where a type is written in an `extern` signature, which is what decides whether it may be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Crossing {
    /// A parameter, which is a value the caller already holds.
    Taken,
    /// The result of a call, which is where nothing given back and the two answers are written.
    GivenBack,
    /// The result of a call whose declaration narrows an argument, where a `None` also says that
    /// an argument did not fit, so an `Option` over a number says something after all.
    Narrowed,
    /// The result of a `field`, which is a value a class holds and so is never nothing at all.
    Held,
}

impl Crossing {
    /// Where the result of `declaration` is written, which a `field` holds rather than gives.
    ///
    /// The JVM has no field of type `void`, so a `field` declared `()` reaches nothing to read,
    /// and refusing it here is what leaves the lowering with a value to get in every case.
    pub(crate) fn of(declaration: &ExternDeclaration) -> Self {
        match &declaration.reaches {
            Reaches::Field(_) => Self::Held,
            Reaches::Static(_) | Reaches::Method(_) | Reaches::New => {
                if declaration.narrows() {
                    Self::Narrowed
                } else {
                    Self::GivenBack
                }
            }
        }
    }

    /// Whether the result of a call is written here, which is where `()` may be.
    const fn of_a_call(self) -> bool {
        matches!(self, Self::GivenBack | Self::Narrowed)
    }

    /// What a member does with a value written here, as the message of `L0425` reads it.
    pub(crate) const fn what_a_member_does(self) -> &'static str {
        match self {
            Self::Taken => "takes",
            Self::GivenBack | Self::Narrowed => "gives back",
            Self::Held => "holds",
        }
    }
}

/// `held` held to what a Java member carries where it is written, or why it is not one.
///
/// # Errors
///
/// Returns a type no Java member takes or gives back in the place it is written.
pub(crate) fn crosses(
    held: &Type,
    crossing: Crossing,
    foreign: &Foreign,
    span: Span,
) -> Result<(), TypeError> {
    if (Boundary { crossing, foreign }).carries(held) {
        return Ok(());
    }
    let kind = ReachingJava::DoesNotCross {
        written: held.clone(),
        crossing,
    };
    Err(TypeError::at(span, kind.into()))
}

/// One place in an `extern` signature, as the reading of what may be written there holds it.
///
/// Where a type is written and which classes the declaring module named are both read to answer
/// whether it crosses, so the two travel together rather than down every call apart.
struct Boundary<'a> {
    crossing: Crossing,
    foreign: &'a Foreign,
}

impl Boundary<'_> {
    /// Whether a Java member carries `held` where this boundary says it is written.
    ///
    /// `()` is the one a `field` parts company over: a member gives nothing back, and a field
    /// holds something or is no field.
    ///
    /// `List<T>` is a `java.util.List` already, so a member takes one as it takes any other
    /// value. It is a parameter and never a result: a list a member gives back is a JVM object
    /// the member may still reach through and change, and a Lumen value is never that. What it
    /// holds is held to the same rule, so a list of a type no member takes is none either.
    fn carries(&self, held: &Type) -> bool {
        let Type::Named { name, arguments } = held else {
            return matches!(held, Type::Unit) && self.crossing.of_a_call();
        };
        match (name.as_str(), arguments.as_slice()) {
            ("Bool" | "Int" | "String", []) => true,
            (LIST, [element]) => self.crossing == Crossing::Taken && self.taken().carries(element),
            (_, []) => self.foreign.contains_key(name),
            _ => self.is_an_answer(held),
        }
    }

    /// Whether `held` is an answer a member gives back, carrying what it wraps where it is.
    ///
    /// `Option` and `Result` are answers rather than values, so each is a result and never a
    /// parameter, and each is read for what it wraps the same way. Neither wraps `()`, because a
    /// variant carrying nothing at all is not a thing a constructor of one builds.
    fn is_an_answer(&self, held: &Type) -> bool {
        let Type::Named { name, arguments } = held else {
            return false;
        };
        if self.crossing == Crossing::Taken {
            return false;
        }
        match (name.as_str(), arguments.as_slice()) {
            (OPTION, [value]) => self.optionally(value),
            (RESULT, [value, Type::Named { name, .. }]) if name == "String" => {
                !matches!(value, Type::Unit) && self.carries(value)
            }
            _ => false,
        }
    }

    /// Whether a `None` says anything about `held` where an `Option` is written around it.
    ///
    /// A `null` is the `None` wherever the member gives back a reference. `Option<Int>` is
    /// otherwise neither a value nor an answer: a `long` is never `null`, so nothing it held
    /// could say `None`. A declaration that narrows an argument has a second reason for one,
    /// which `docs/specs/interop.md` states, so there a number says something after all.
    fn optionally(&self, held: &Type) -> bool {
        is_a_reference(held, self.foreign)
            || (self.crossing == Crossing::Narrowed && is_a_number(held))
    }

    /// The same classes, read where a member takes a value, which an element of a list is.
    const fn taken(&self) -> Self {
        Self {
            crossing: Crossing::Taken,
            foreign: self.foreign,
        }
    }
}

/// The class `declaration` reaches, held to being one its signature names.
///
/// A `method` is called on its receiver and a `new` builds what it gives back, so each of the
/// two names its class in the signature rather than in the string. A signature naming none
/// leaves the declaration with no class to reach, which `docs/specs/interop.md` refuses.
///
/// # Errors
///
/// Returns the kind whose class the signature does not name.
pub(crate) fn reaches_a_class(
    declaration: &ExternDeclaration,
    taken: &[Type],
    result: &Type,
    foreign: &Foreign,
) -> Result<(), TypeError> {
    let (reached, span) = match &declaration.reaches {
        Reaches::Method(_) => (taken.first(), receiver_of(declaration)),
        Reaches::New => (Some(given_back(result)), declaration.result.span),
        Reaches::Field(_) | Reaches::Static(_) => return Ok(()),
    };
    if reached.is_some_and(|held| is_a_reference(held, foreign)) {
        return Ok(());
    }
    let kind = ReachingJava::ReachesNoClass(declaration.reaches.written().to_owned());
    Err(TypeError::at(span, kind.into()))
}

/// The class a `new` builds, held to being a class rather than an interface.
///
/// Every other kind reaches a member, and an interface has members. A constructor is the one
/// thing an interface does not have, so a `new` giving one back is a class file that will not
/// link, and `docs/specs/interop.md` refuses it where it is written instead.
///
/// # Errors
///
/// Returns the type a `new` gives back, where an `extern type` named it an interface.
pub(crate) fn builds_a_class(
    declaration: &ExternDeclaration,
    result: &Type,
    foreign: &Foreign,
) -> Result<(), TypeError> {
    let built = given_back(result);
    if declaration.reaches != Reaches::New || !is_an_interface(built, foreign) {
        return Ok(());
    }
    let kind = ReachingJava::BuildsAnInterface(built.clone());
    Err(TypeError::at(declaration.result.span, kind.into()))
}

/// Whether `held` is a type an `extern type` named an interface.
fn is_an_interface(held: &Type, foreign: &Foreign) -> bool {
    let Type::Named { name, .. } = held else {
        return false;
    };
    foreign.get(name) == Some(&Called::AsAnInterface)
}

/// The width `declaration` writes, held to a result there is an `Int` to widen an `int` to.
///
/// One rule holds for every kind. A `new` meets it the way any other does, because the class a
/// constructor gives back is not `Int`. `docs/specs/interop.md` states the widening.
///
/// # Errors
///
/// Returns what a declaration written `int` gives back, where that is not an `Int`.
pub(crate) fn widens(declaration: &ExternDeclaration, result: &Type) -> Result<(), TypeError> {
    let Some(width) = declaration.gives.written() else {
        return Ok(());
    };
    let given = given_back(result);
    if is_an_int(given) {
        return Ok(());
    }
    let kind = ReachingJava::WidensNoInt {
        width: width.to_owned(),
        written: given.clone(),
    };
    Err(TypeError::at(declaration.result.span, kind.into()))
}

/// The width `parameter` writes, held to a type there is an `Int` to narrow to an `int`.
///
/// # Errors
///
/// Returns what a parameter written `int` takes, where that is not an `Int`.
pub(crate) fn narrowed(parameter: &ExternParameter, held: &Type) -> Result<(), TypeError> {
    if parameter.takes == Takes::WhatTheParameterIs || is_an_int(held) {
        return Ok(());
    }
    let kind = ReachingJava::NarrowsNoInt(held.clone());
    Err(TypeError::at(parameter.declared.name.span, kind.into()))
}

/// A declaration that narrows an argument, held to a result there is a `None` to answer with.
///
/// Narrowing a `long` to an `int` loses whatever does not fit, and nothing in Bux is partial, so
/// an argument outside the `int` range is a `None`. A `Result` around the result is read through,
/// because the guard is outside and such an argument is an `Ok(None)`.
///
/// # Errors
///
/// Returns what a declaration that narrows an argument gives back, where that is no `Option`.
pub(crate) fn narrows(declaration: &ExternDeclaration, result: &Type) -> Result<(), TypeError> {
    let given = inside(RESULT, result);
    if !declaration.narrows() || is_optional(given) {
        return Ok(());
    }
    let kind = ReachingJava::NarrowsWithoutOption(given.clone());
    Err(TypeError::at(declaration.result.span, kind.into()))
}

/// Whether `held` is the `Option` a `None` is one answer of.
fn is_optional(held: &Type) -> bool {
    matches!(held, Type::Named { name, arguments } if name == OPTION && arguments.len() == 1)
}

/// Whether the JVM holds a value of `held` as a number, which is what no `null` is.
fn is_a_number(held: &Type) -> bool {
    matches!(held, Type::Named { name, arguments }
        if arguments.is_empty() && (name == "Bool" || name == "Int"))
}

/// Whether `held` is the `Int` a widened `int` becomes.
fn is_an_int(held: &Type) -> bool {
    matches!(held, Type::Named { name, arguments } if name == "Int" && arguments.is_empty())
}

/// Where the receiver of a `method` is written, which is the whole declaration where none is.
fn receiver_of(declaration: &ExternDeclaration) -> Span {
    declaration
        .parameters
        .first()
        .map_or(declaration.span, |parameter| parameter.declared.span)
}

/// What the member's own descriptor gives back, which is the result with its answers read off.
fn given_back(result: &Type) -> &Type {
    inside(OPTION, inside(RESULT, result))
}

/// What `wrapper` wraps where `held` is one, and `held` itself where it is not.
fn inside<'a>(wrapper: &str, held: &'a Type) -> &'a Type {
    match held {
        Type::Named { name, arguments } if name == wrapper => arguments.first().unwrap_or(held),
        _ => held,
    }
}

/// Whether a value of `held` is one the JVM holds as a reference, which is what may be `null`.
fn is_a_reference(held: &Type, foreign: &Foreign) -> bool {
    matches!(held, Type::Named { name, arguments }
        if arguments.is_empty() && (name == "String" || foreign.contains_key(name)))
}

/// The Java name `declaration` states, held to what its kind names.
///
/// # Errors
///
/// Returns a name that is no Java name, or that names more or fewer segments than its kind does.
pub(crate) fn stated_by(declaration: &ExternDeclaration) -> Result<(), TypeError> {
    let wanted = match &declaration.reaches {
        // The last segment is the member, and everything before it is the class it is on.
        Reaches::Field(named) | Reaches::Static(named) => return at_least(named, 2),
        // The receiver's type already says which class it is on, so only the member is named.
        Reaches::Method(named) => named,
        Reaches::New => return Ok(()),
    };
    exactly(wanted, 1)
}

/// The class an `extern type` names, which is a Java name of one segment or more.
///
/// # Errors
///
/// Returns a name that is no Java name.
pub(crate) fn class(named: &JavaName) -> Result<(), TypeError> {
    at_least(named, 1)
}

/// `named` held to being a Java name of `least` segments or more.
fn at_least(named: &JavaName, least: usize) -> Result<(), TypeError> {
    held_to(named, |segments| segments >= least)
}

/// `named` held to being a Java name of exactly `count` segments.
fn exactly(named: &JavaName, count: usize) -> Result<(), TypeError> {
    held_to(named, |segments| segments == count)
}

/// `named` held to being a Java name whose segment count `enough` accepts.
fn held_to(named: &JavaName, enough: impl Fn(usize) -> bool) -> Result<(), TypeError> {
    let segments = named.segments();
    if enough(segments.len()) && segments.iter().all(|segment| is_a_segment(segment)) {
        return Ok(());
    }
    let kind = ReachingJava::NotAJavaName(named.text.clone());
    Err(TypeError::at(named.span, kind.into()))
}

/// Whether `segment` is one segment of a Java name, which is what Java calls an identifier.
fn is_a_segment(segment: &str) -> bool {
    let mut characters = segment.chars();
    characters.next().is_some_and(opens_a_segment) && characters.all(continues_a_segment)
}

/// Whether a Java name may begin with `character`, which a digit never does.
fn opens_a_segment(character: char) -> bool {
    character.is_alphabetic() || character == '_' || character == '$'
}

/// Whether a Java name may go on with `character`.
fn continues_a_segment(character: char) -> bool {
    character.is_alphanumeric() || character == '_' || character == '$'
}
