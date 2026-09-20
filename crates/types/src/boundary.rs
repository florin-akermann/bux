//! What crosses the boundary an `extern` declaration is, and what a Java name is.
//!
//! `docs/specs/interop.md` states both. A signature is the whole of the boundary: every type in
//! it compiles to exactly the JVM type the member's own descriptor holds, so the types that may
//! be written are the ones that have one. What a declaration cannot say is not reachable, and
//! the answer to that is another declaration rather than a rule that widens one of these.

use std::collections::HashMap;

use lumen_ast::{ExternDeclaration, JavaName, Reaches, Span};

use crate::error::{TypeError, TypeErrorKind};
use crate::types::{OPTION, RESULT, Type};

/// The Java class each extern type stands for, by the Lumen name an `extern type` gives it.
pub(crate) type Foreign = HashMap<String, String>;

/// Where a type is written in an `extern` signature, which is what decides whether it may be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Crossing {
    /// A parameter, which is a value the caller already holds.
    Taken,
    /// The result of a call, which is where nothing given back and the two answers are written.
    GivenBack,
    /// The result of a `field`, which is a value a class holds and so is never nothing at all.
    Held,
}

impl Crossing {
    /// Where the result of `reaches` is written, which a `field` alone holds rather than gives.
    ///
    /// The JVM has no field of type `void`, so a `field` declared `()` reaches nothing to read,
    /// and refusing it here is what leaves the lowering with a value to get in every case.
    pub(crate) const fn of(reaches: &Reaches) -> Self {
        match reaches {
            Reaches::Field(_) => Self::Held,
            Reaches::Static(_) | Reaches::Method(_) | Reaches::New => Self::GivenBack,
        }
    }

    /// What a member does with a value written here, as the message of `L0425` reads it.
    pub(crate) const fn what_a_member_does(self) -> &'static str {
        match self {
            Self::Taken => "takes",
            Self::GivenBack => "gives back",
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
    if carries(held, crossing, foreign) {
        return Ok(());
    }
    let kind = TypeErrorKind::DoesNotCross {
        written: held.clone(),
        crossing,
    };
    Err(TypeError::at(span, kind))
}

/// Whether a Java member carries `held` where `crossing` says it is written.
///
/// `Option` and `Result` are answers rather than values, so each is a result and never a
/// parameter, and each is read for what it wraps the same way. `Option<Int>` is neither: a
/// `long` is never `null`, so nothing it held could say `None`. Neither wraps `()` either,
/// because a variant carrying nothing at all is not a thing a constructor of one builds.
/// `()` itself is the one a `field` parts company over: a member gives nothing back, and a
/// field holds something or is no field.
fn carries(held: &Type, crossing: Crossing, foreign: &Foreign) -> bool {
    let Type::Named { name, arguments } = held else {
        return matches!(held, Type::Unit) && crossing == Crossing::GivenBack;
    };
    let a_result = crossing != Crossing::Taken;
    match (name.as_str(), arguments.as_slice()) {
        ("Bool" | "Int" | "String", []) => true,
        (OPTION, [value]) if a_result => is_a_reference(value, foreign),
        (RESULT, [value, Type::Named { name, .. }]) if a_result && name == "String" => {
            !matches!(value, Type::Unit) && carries(value, crossing, foreign)
        }
        (_, []) => foreign.contains_key(name),
        _ => false,
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
    let kind = TypeErrorKind::ReachesNoClass(declaration.reaches.written().to_owned());
    Err(TypeError::at(span, kind))
}

/// Where the receiver of a `method` is written, which is the whole declaration where none is.
fn receiver_of(declaration: &ExternDeclaration) -> Span {
    declaration
        .parameters
        .first()
        .map_or(declaration.span, |parameter| parameter.span)
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
    let kind = TypeErrorKind::NotAJavaName(named.text.clone());
    Err(TypeError::at(named.span, kind))
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
