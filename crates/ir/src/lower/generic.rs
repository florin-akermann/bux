//! A use of a generic function, which is what decides the method that use reaches.
//!
//! `docs/specs/codegen.md` writes a generic function once per set of types it is used at. This is
//! what a set of types is here: what each type parameter the function declares settled on, read
//! off the type inference gave the use.

use std::collections::HashMap;
use std::fmt::Write as _;

use lumen_ast::Function;
use lumen_resolver::Origin;
use lumen_types::Type;

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
    settled: Vec<(Origin, Type)>,
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
        let settled = function
            .type_parameters
            .iter()
            .map(|written| {
                let origin = Origin::Declared(written.span);
                let at = found.get(&origin).cloned();
                (origin, at.unwrap_or_else(|| itself(written)))
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
    ///
    /// `$` is legal in a method name and Lumen writes no operator with it, so a name reached
    /// this way is one no source collides with.
    pub(crate) fn names(&self, function: &str) -> String {
        self.settled
            .iter()
            .fold(function.to_owned(), |mut named, (_, at)| {
                let _ = write!(named, "${}", head_of(at));
                named
            })
    }

    fn at(&self, parameter: Origin) -> Option<Type> {
        self.settled
            .iter()
            .find(|(origin, _)| *origin == parameter)
            .map(|(_, at)| at.clone())
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

/// The name a type gives the method written for a use that settled on it.
///
/// A type argument never reaches a descriptor, so a type is named by its own name however it is
/// written: `Option<Int>` and `Option<Bool>` are both `Option`, and both need the one method.
fn head_of(at: &Type) -> &str {
    match at {
        Type::Named { name, .. } => name,
        Type::Unit => NOTHING_AT_ALL,
        Type::Var(_) | Type::Parameter(_) | Type::Function { .. } | Type::Module(_) => {
            SETTLED_NOTHING
        }
    }
}

/// The type parameter `written` standing for itself, which is what an unsettled one does.
fn itself(written: &lumen_ast::Name) -> Type {
    Type::Parameter(lumen_types::TypeParameter::written(written))
}
