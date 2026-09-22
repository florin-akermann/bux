//! A use of a generic function, which is what decides the method that use reaches.
//!
//! `docs/specs/codegen.md` writes a generic function once per set of types it is used at. This is
//! what a set of types is here: what each type parameter the function declares settled on, read
//! off the type inference gave the use.

use std::collections::HashMap;
use std::fmt::Write as _;

use lumen_ast::Function;
use lumen_resolver::Origin;
use lumen_types::{Type, TypeParameter};

/// What a type argument that settled nothing is called, which is what `Object` carries.
const SETTLED_NOTHING: &str = "Any";

/// What `()` is called, which is a type a parameter settles on like any other.
const NOTHING_AT_ALL: &str = "Unit";

/// What one use of a function settled each type parameter it declares at, in declared order.
///
/// A function that declares none has the one that settles nothing, and everything below it reads
/// as it did before generics: a type is carried by itself, and the method is named as written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Instantiation {
    settled: Vec<Settled>,
}

/// What one use settled one type parameter on, and what that type is called in a method name.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Settled {
    origin: Origin,
    at: Type,
    /// Whether the declaration constrains the parameter, which is what reaches an instance of it.
    constrained: bool,
}

impl Instantiation {
    /// The use a function declaring no type parameter has, which settles nothing.
    pub(crate) const fn whole() -> Self {
        Self {
            settled: Vec::new(),
        }
    }

    /// What a use settled, where `declared` is the function's type and `used` is the use's.
    ///
    /// A type parameter the signature never writes is settled by nothing and stands for itself,
    /// which is what `Any` names and what `Object` carries.
    pub(crate) fn of(function: &Function, declared: &Type, used: &Type) -> Self {
        let mut found = HashMap::new();
        settle(declared, used, &mut found);
        let at = |written: &lumen_ast::Name| {
            let origin = Origin::Declared(written.span);
            found.get(&origin).cloned()
        };
        Self::over(function, at)
    }

    /// A use that settled `settled`, in the order `function` declares its type parameters.
    ///
    /// This is the use another module asked for: it settled the set where it is written, and
    /// `docs/specs/codegen.md` has the module declaring the function write the method for it.
    pub(crate) fn asked_for(function: &Function, settled: &[Type]) -> Self {
        let at = |written: &lumen_ast::Name| {
            let position = function
                .type_parameters
                .iter()
                .position(|declared| declared.name.span == written.span);
            position.and_then(|position| settled.get(position).cloned())
        };
        Self::over(function, at)
    }

    /// One use of `function`, with `at` saying what each type parameter it declares settled on.
    fn over(function: &Function, at: impl Fn(&lumen_ast::Name) -> Option<Type>) -> Self {
        let settled = function
            .type_parameters
            .iter()
            .map(|written| Settled {
                origin: Origin::Declared(written.name.span),
                at: at(&written.name).unwrap_or_else(|| itself(&written.name)),
                constrained: written.constraint.is_some(),
            })
            .collect();
        Self { settled }
    }

    /// `of` with each type parameter this settled written as the type it settled on.
    ///
    /// This is what makes a body read at the types its use settled: every type in it is asked
    /// for through here, so a parameter is carried by whatever the use carried it by.
    pub(crate) fn substituted(&self, of: &Type) -> Type {
        if self.settled.is_empty() {
            return of.clone();
        }
        match of {
            Type::Parameter(parameter) => self.at(parameter.origin).unwrap_or_else(|| of.clone()),
            Type::Named { name, arguments } => Type::Named {
                name: name.clone(),
                arguments: arguments.iter().map(|at| self.substituted(at)).collect(),
            },
            Type::Function { parameters, result } => Type::Function {
                parameters: parameters.iter().map(|of| self.substituted(of)).collect(),
                result: Box::new(self.substituted(result)),
            },
            Type::Var(_) | Type::Module(_) | Type::Unit => of.clone(),
        }
    }

    /// What the method written for this use is called: the function, then each type it settled.
    pub(crate) fn names(&self, function: &str) -> String {
        self.settled
            .iter()
            .fold(function.to_owned(), |mut named, settled| {
                let _ = write!(named, "${}", named_at(&settled.at, settled.constrained));
                named
            })
    }

    fn at(&self, parameter: Origin) -> Option<Type> {
        self.settled
            .iter()
            .find(|settled| settled.origin == parameter)
            .map(|settled| settled.at.clone())
    }
}

/// Reads off `used` what each type parameter `declared` writes settled on.
///
/// The two are the same shape, because `used` is `declared` with each parameter settled, so
/// walking them together is all it takes. Anywhere they are not, there is nothing to read.
fn settle(declared: &Type, used: &Type, into: &mut HashMap<Origin, Type>) {
    match (declared, used) {
        (Type::Parameter(parameter), settled) => {
            into.entry(parameter.origin)
                .or_insert_with(|| settled.clone());
        }
        (
            Type::Named {
                arguments: mine, ..
            },
            Type::Named {
                arguments: theirs, ..
            },
        ) => {
            for (mine, theirs) in mine.iter().zip(theirs) {
                settle(mine, theirs, into);
            }
        }
        (
            Type::Function {
                parameters: mine,
                result: my_result,
            },
            Type::Function {
                parameters: theirs,
                result: their_result,
            },
        ) => {
            for (mine, theirs) in mine.iter().zip(theirs) {
                settle(mine, theirs, into);
            }
            settle(my_result, their_result, into);
        }
        _ => {}
    }
}

/// What the method written for a use that settled `at` is called.
///
/// The function's own name, then each type the use settled a type parameter on, joined by `$`.
/// `$` is legal in a method name and Lumen writes no operator with it, so a name reached this way
/// is one no source collides with. A module asking another for a method names it the same way,
/// which is what has the two agree without either reading the other's tree.
pub(crate) fn names(function: &str, at: &[Type], constrained: &[Option<String>]) -> String {
    at.iter()
        .zip(constrained)
        .fold(function.to_owned(), |mut named, (at, of)| {
            let _ = write!(named, "${}", named_at(at, of.is_some()));
            named
        })
}

/// What one type a use settled a type parameter on is called inside a method name.
///
/// A constrained parameter reaches the instance of the whole type it settled on, and two types
/// with one head have two instances, so the whole of it is what tells the two methods apart. An
/// unconstrained one reaches no instance, so its head is all a name needs, and that is what has a
/// generic calling itself at a type one deeper ask for a method already written.
fn named_at(at: &Type, constrained: bool) -> String {
    if constrained {
        return wholly(at);
    }
    head_of(at)
}

/// What the method an instance over a type written with type parameters writes is called.
///
/// Every argument is written out, nested ones included, because such an instance hands each
/// value it holds to the instance of that value's own type: `List<Int>` and `List<Bool>` are two
/// methods, which naming by the head alone would give one name and one body.
pub(crate) fn names_wholly(named: &str, at: &[Type]) -> String {
    at.iter().fold(named.to_owned(), |mut named, at| {
        let _ = write!(named, "${}", wholly(at));
        named
    })
}

/// One type written out whole: its own name, and then every argument it is written with.
fn wholly(at: &Type) -> String {
    let Type::Named { name, arguments } = at else {
        return head_of(at);
    };
    arguments
        .iter()
        .fold(name.replace('.', "$"), |mut written, argument| {
            let _ = write!(written, "${}", wholly(argument));
            written
        })
}

/// The name a type gives the method written for a use that settled on it.
///
/// A type argument never reaches a descriptor, so a type is named by its own name however it is
/// written: `Option<Int>` and `Option<Bool>` are both `Option`, and both need the one method.
/// A type of another module is written `demo.User` and named `demo$User`, because a JVM method
/// name holds no dot.
fn head_of(at: &Type) -> String {
    match at {
        Type::Named { name, .. } => name.replace('.', "$"),
        Type::Unit => NOTHING_AT_ALL.to_owned(),
        Type::Var(_) | Type::Parameter(_) | Type::Function { .. } | Type::Module(_) => {
            SETTLED_NOTHING.to_owned()
        }
    }
}

/// The type parameter `written` standing for itself, which is what an unsettled one does.
fn itself(written: &lumen_ast::Name) -> Type {
    Type::Parameter(TypeParameter::written(written))
}
