//! Type inference: every expression of a module given the type it has.
//!
//! The phase consumes the resolved tree and yields a [`TypedProgram`], where each expression
//! carries the type inference gave it. Nothing is mutated: the tree comes through unchanged and
//! the answers sit beside it. `docs/specs/types.md` is the specification, and `docs/design.md`
//! sections 3, 6, and 7 are the rules it enforces.

mod bounds;
mod derive;
mod environment;
mod error;
mod holds;
mod infer;
mod scheme;
mod supplied;
mod surface;
mod table;
mod types;
mod unify;

use std::collections::HashMap;

use lumen_ast::Span;
use lumen_resolver::ResolvedProgram;

pub use crate::error::TypeError;
pub use crate::supplied::supplies;
pub use crate::surface::{Imported, Surface};
pub use crate::types::{Type, TypeParameter, TypeVar};

/// Gives every expression of `resolved` the type it has, reaching `imported` through an import.
///
/// # Errors
///
/// Returns the first declaration that holds a value of itself, or, where none does, the first
/// expression whose type inference cannot give it.
pub fn check(resolved: ResolvedProgram, imported: &Imported) -> Result<TypedProgram, TypeError> {
    holds::nothing_holds_itself(&resolved)?;
    let inferred = infer::infer(&resolved, imported)?;
    Ok(TypedProgram {
        resolved,
        types: inferred.types,
        surface: inferred.surface,
        methods: inferred.methods,
    })
}

/// A program whose every expression, and every name that declares one, has the type it has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedProgram {
    resolved: ResolvedProgram,
    types: HashMap<Span, Type>,
    surface: Surface,
    methods: HashMap<Span, Type>,
}

impl TypedProgram {
    /// The program this was inferred from, with every name still pointed at its definition.
    #[must_use]
    pub const fn resolved(&self) -> &ResolvedProgram {
        &self.resolved
    }

    /// What this module offers the modules that import it.
    #[must_use]
    pub const fn surface(&self) -> &Surface {
        &self.surface
    }

    /// The type of what is written at `at`.
    ///
    /// Every expression has one, and so does every name that declares something: a function, a
    /// parameter, and each name a binding or a pattern introduces.
    #[must_use]
    pub fn type_of(&self, written: Span) -> Option<&Type> {
        self.types.get(&written)
    }

    /// The type a use of a trait method at `written` reached its instance at.
    ///
    /// Every use of a trait method has one, and nothing else does: that is what tells lowering
    /// which instance the call is a call of, which `docs/specs/traits.md` states is settled
    /// while the program is compiled.
    #[must_use]
    pub fn instance_at(&self, written: Span) -> Option<&Type> {
        self.methods.get(&written)
    }
}
