//! Name resolution: every name of a module pointed at the definition it means.
//!
//! The phase consumes the untyped tree the parser produced and yields a [`ResolvedProgram`],
//! where each name that has a definition carries which one. Nothing is mutated: the tree comes
//! through unchanged and the answers sit beside it. `docs/specs/modules.md` is the specification,
//! and `docs/design.md` section 16 is the rule it enforces: one name has one definition.

mod definition;
mod error;
mod order;
pub mod prelude;
mod resolve;
mod scope;

pub use definition::{Definition, DefinitionKind, Namespace, Origin};
pub use error::ResolveError;
pub use resolve::{ResolvedProgram, prelude_resolved, resolve};
