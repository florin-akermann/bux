//! Type inference: every expression of a module given the type it has.
//!
//! The phase consumes the resolved tree and yields a [`TypedProgram`], where each expression
//! carries the type inference gave it. Nothing is mutated: the tree comes through unchanged and
//! the answers sit beside it. `docs/specs/types.md` is the specification, and `docs/design.md`
//! sections 3, 6, and 7 are the rules it enforces.

mod environment;
mod error;
mod holds;
mod infer;
mod scheme;
mod supplied;
mod table;
mod types;
mod unify;

use std::collections::HashMap;

use lumen_ast::Span;
use lumen_resolver::ResolvedProgram;

pub use crate::error::TypeError;
pub use crate::types::{Type, TypeParameter, TypeVar};

/// Gives every expression of `resolved` the type it has.
///
/// # Errors
///
/// Returns the first declaration that holds a value of itself, or, where none does, the first
/// expression whose type inference cannot give it.
pub fn check(resolved: ResolvedProgram) -> Result<TypedProgram, TypeError> {
    holds::nothing_holds_itself(&resolved)?;
    let types = infer::infer(&resolved)?;
    Ok(TypedProgram { resolved, types })
}

/// A program whose every expression, and every name that declares one, has the type it has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedProgram {
    resolved: ResolvedProgram,
    types: HashMap<Span, Type>,
}

impl TypedProgram {
    /// The program this was inferred from, with every name still pointed at its definition.
    #[must_use]
    pub const fn resolved(&self) -> &ResolvedProgram {
        &self.resolved
    }

    /// The type of what is written at `at`.
    ///
    /// Every expression has one, and so does every name that declares something: a function, a
    /// parameter, and each name a binding or a pattern introduces.
    #[must_use]
    pub fn type_of(&self, written: Span) -> Option<&Type> {
        self.types.get(&written)
    }
}
