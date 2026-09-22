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
        self.at_one_use(table).found
    }

    /// This scheme at one use: the type it has there, what it asks for, and what it settled.
    pub(crate) fn at_one_use(&self, table: &mut Table) -> AtOneUse {
        let given: HashMap<Quantified, Type> = self
            .quantified
            .iter()
            .map(|one| (one.clone(), table.fresh()))
            .collect();
        let required = self
            .required
            .iter()
            .map(|one| Required {
                trait_name: one.trait_name.clone(),
                at: replaced(&one.at, &given),
            })
            .collect();
        AtOneUse {
            found: replaced(&self.body, &given),
            required,
            settling: self.written_stand_ins(&given),
            constrained: self.constrained_stand_ins(),
            written_as: replaced(&self.body, &erasing(&given)),
        }
    }

    /// What this use gave each stand-in the declaration wrote, in the order it wrote them.
    ///
    /// A stand-in inference made is not among them: `docs/specs/codegen.md` writes a generic once
    /// per set of types its written type parameters settled on, and names it after those alone.
    fn written_stand_ins(&self, given: &HashMap<Quantified, Type>) -> Vec<Type> {
        self.quantified
            .iter()
            .filter(|one| matches!(one, Quantified::Parameter(_)))
            .filter_map(|one| given.get(one).cloned())
            .collect()
    }

    /// The trait constraining each stand-in the declaration wrote, in the order it wrote them.
    ///
    /// A constrained one reaches the instance of whatever it settled on, so `docs/specs/codegen.md`
    /// names the method written for it after the whole of that type rather than after its head,
    /// and has the module that settled the type write the instance method that method calls.
    fn constrained_stand_ins(&self) -> Vec<Option<String>> {
        self.quantified
            .iter()
            .filter_map(|one| match one {
                Quantified::Parameter(parameter) => Some(self.required_of(parameter)),
                Quantified::Var(_) => None,
            })
            .collect()
    }

    /// The trait one of this scheme's constraints is written over `parameter` by, where one is.
    fn required_of(&self, parameter: &TypeParameter) -> Option<String> {
        let written = Type::Parameter(parameter.clone());
        self.required
            .iter()
            .find(|required| required.at == written)
            .map(|required| required.trait_name.clone())
    }

    /// This scheme with `one` of its stand-ins settled on `given`, which is what an instance is.
    pub(crate) fn settling(&self, one: &Quantified, given: Type) -> Type {
        replaced(&self.body, &HashMap::from([(one.clone(), given)]))
    }

    /// The same scheme, with `rename` applied to every type it holds.
    ///
    /// A module offering its surface writes each of its own types as a name reached through it,
    /// and a scheme is what a function offers, so this is how one crosses that boundary.
    pub(crate) fn renamed(&self, rename: &impl Fn(&Type) -> Type) -> Self {
        Self {
            quantified: self.quantified.clone(),
            body: rename(&self.body),
            required: self
                .required
                .iter()
                .map(|one| Required {
                    trait_name: one.trait_name.clone(),
                    at: rename(&one.at),
                })
                .collect(),
        }
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

/// One use of a scheme: the type it has there, what it asks for, and what it settled.
pub(crate) struct AtOneUse {
    pub(crate) found: Type,
    /// Each trait the use must answer for, at the type this use settled it at.
    pub(crate) required: Vec<Required>,
    /// What each stand-in the declaration wrote settled on, in the order it wrote them.
    pub(crate) settling: Vec<Type>,
    /// The trait constraining each of those stand-ins, in the same order.
    pub(crate) constrained: Vec<Option<String>>,
    /// The type the method written for this use has, which is not the type the use has.
    ///
    /// `docs/specs/codegen.md` writes a generic at the types its written type parameters settled
    /// on and at nothing else, so a stand-in inference made is carried by whatever every value
    /// fits. A use of `fn passed(value) { value }` at an `Int` therefore reaches a method that
    /// takes and gives back what a type parameter is carried by, not one that takes an `Int`.
    pub(crate) written_as: Type,
}

/// `given`, with every stand-in inference made standing for the one type every value fits.
fn erasing(given: &HashMap<Quantified, Type>) -> HashMap<Quantified, Type> {
    given
        .iter()
        .map(|(one, at)| match one {
            Quantified::Var(_) => (one.clone(), Type::Parameter(TypeParameter::erased())),
            Quantified::Parameter(_) => (one.clone(), at.clone()),
        })
        .collect()
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
