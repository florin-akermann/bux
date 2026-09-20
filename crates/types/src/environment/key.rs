//! What a name is filed under, and the stand-ins a declaration is written over.
//!
//! One flat table holds a whole module, so a name is known by where it was defined rather than by
//! how it is spelled: name resolution has already ruled out two definitions sharing a name.

use lumen_ast::{Name, Span};
use lumen_resolver::{Definition, Origin};

use crate::scheme::Quantified;
use crate::types::{Type, TypeParameter};

/// Where a name was defined, which is what a type is filed under.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Key {
    /// A name this module declares, known by the place it is declared.
    Declared(Span),
    /// A name the prelude supplies, known by the one spelling it has.
    Prelude(String),
    /// A name another module declares, known by the name this module reaches it through.
    Reached(String),
}

impl Key {
    /// The key of a use of `name`, which `definition` says where to find.
    pub(crate) fn of(definition: Definition, name: &Name) -> Self {
        match definition.origin {
            Origin::Declared(span) => Self::Declared(span),
            Origin::Prelude => Self::Prelude(name.text.clone()),
        }
    }

    /// The key of the declaration `name` is the name of.
    pub(crate) const fn at(name: &Name) -> Self {
        Self::Declared(name.span)
    }

    /// The key of a name another module declares, which is `demo.User` as this module writes it.
    pub(crate) fn reached(name: &str) -> Self {
        Self::Reached(name.to_owned())
    }

    pub(super) fn prelude(name: &str) -> Self {
        Self::Prelude(name.to_owned())
    }
}

/// One constructor: where it is declared, what it labels, and what it carries.
pub(super) struct Built {
    pub(super) key: Key,
    pub(super) labels: Vec<String>,
    pub(super) carries: Vec<Type>,
}

pub(super) fn quantified(parameters: &[Name]) -> Vec<Quantified> {
    parameters
        .iter()
        .map(|parameter| Quantified::Parameter(TypeParameter::written(parameter)))
        .collect()
}

/// The stand-ins a function is written over, which are its type parameters constrained or not.
pub(super) fn written_over(parameters: &[lumen_ast::TypeParameter]) -> Vec<Quantified> {
    parameters
        .iter()
        .map(|parameter| Quantified::Parameter(TypeParameter::written(&parameter.name)))
        .collect()
}

pub(super) fn parameter_of(definition: Definition, name: &Name) -> TypeParameter {
    TypeParameter {
        name: name.text.clone(),
        origin: definition.origin,
    }
}
