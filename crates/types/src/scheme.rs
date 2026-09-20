//! A type, and what it stands for freshly at each use.

use std::collections::HashMap;

use crate::table::Table;
use crate::types::{Type, TypeParameter, TypeVar};

/// A type together with the stand-ins it is polymorphic over.
///
/// `fn identity<T>(value: T) -> T` is a scheme over one parameter, so each call of it is free to
/// use a type of its own. `docs/specs/types.md` says which declarations get one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Scheme {
    quantified: Vec<Quantified>,
    body: Type,
    /// The traits a use of this must answer for, each written over what this quantifies.
    required: Vec<Required>,
}

/// One constraint a declaration wrote: a trait, and the type it is asked of.
///
/// `fn holds<T: Eq<T>>` requires `Eq` of `T`, so every use of `holds` requires it of whatever
/// that use settled `T` on; `docs/specs/traits.md` states the rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Required {
    pub(crate) trait_name: String,
    pub(crate) at: Type,
}

impl Scheme {
    /// A type that is the same type wherever it is used.
    pub(crate) const fn monomorphic(body: Type) -> Self {
        Self {
            quantified: Vec::new(),
            body,
            required: Vec::new(),
        }
    }

    /// A type that is fresh at each use, over the stand-ins `quantified` lists.
    pub(crate) const fn over(quantified: Vec<Quantified>, body: Type) -> Self {
        Self {
            quantified,
            body,
            required: Vec::new(),
        }
    }

    /// The same scheme, with the traits a use of it must answer for.
    pub(crate) fn requiring(mut self, required: Vec<Required>) -> Self {
        self.required = required;
        self
    }

    /// This scheme at one use: every stand-in replaced by a variable of its own.
    pub(crate) fn instantiate(&self, table: &mut Table) -> Type {
        self.at_one_use(table).0
    }

    /// This scheme at one use, and each trait that use must answer for, at the type it settled.
    pub(crate) fn at_one_use(&self, table: &mut Table) -> (Type, Vec<Required>) {
        let given: HashMap<Quantified, Type> = self
            .quantified
            .iter()
            .map(|one| (one.clone(), table.fresh()))
            .collect();
        let asked = self
            .required
            .iter()
            .map(|one| Required {
                trait_name: one.trait_name.clone(),
                at: replaced(&one.at, &given),
            })
            .collect();
        (replaced(&self.body, &given), asked)
    }

    /// This scheme with `one` of its stand-ins settled on `given`, which is what an instance is.
    pub(crate) fn settling(&self, one: &Quantified, given: Type) -> Type {
        replaced(&self.body, &HashMap::from([(one.clone(), given)]))
    }

    /// The traits a use of this must answer for, still written over what this quantifies.
    pub(crate) fn required(&self) -> &[Required] {
        &self.required
    }

    /// The type this quantifies, with its stand-ins still standing in.
    pub(crate) const fn body(&self) -> &Type {
        &self.body
    }

    /// The stand-ins this is polymorphic over.
    pub(crate) fn quantified(&self) -> &[Quantified] {
        &self.quantified
    }
}

/// One stand-in a scheme is polymorphic over.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Quantified {
    /// A variable inference made and then found nothing constrained.
    Var(TypeVar),
    /// A parameter a declaration wrote between its angle brackets.
    Parameter(TypeParameter),
}

/// `within` with every stand-in `given` names replaced by what it is given.
fn replaced(within: &Type, given: &HashMap<Quantified, Type>) -> Type {
    match within {
        Type::Var(var) => stood_in_for(&Quantified::Var(*var), within, given),
        Type::Parameter(parameter) => {
            stood_in_for(&Quantified::Parameter(parameter.clone()), within, given)
        }
        Type::Named { name, arguments } => Type::Named {
            name: name.clone(),
            arguments: each(arguments, given),
        },
        Type::Function { parameters, result } => {
            Type::function(each(parameters, given), replaced(result, given))
        }
        Type::Module(_) | Type::Unit => within.clone(),
    }
}

fn stood_in_for(one: &Quantified, within: &Type, given: &HashMap<Quantified, Type>) -> Type {
    given.get(one).cloned().unwrap_or_else(|| within.clone())
}

fn each(types: &[Type], given: &HashMap<Quantified, Type>) -> Vec<Type> {
    types.iter().map(|one| replaced(one, given)).collect()
}
