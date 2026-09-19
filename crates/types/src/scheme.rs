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
}

impl Scheme {
    /// A type that is the same type wherever it is used.
    pub(crate) const fn monomorphic(body: Type) -> Self {
        Self {
            quantified: Vec::new(),
            body,
        }
    }

    /// A type that is fresh at each use, over the stand-ins `quantified` lists.
    pub(crate) const fn over(quantified: Vec<Quantified>, body: Type) -> Self {
        Self { quantified, body }
    }

    /// This scheme at one use: every stand-in replaced by a variable of its own.
    pub(crate) fn instantiate(&self, table: &mut Table) -> Type {
        let given: HashMap<Quantified, Type> = self
            .quantified
            .iter()
            .map(|one| (one.clone(), table.fresh()))
            .collect();
        replaced(&self.body, &given)
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
