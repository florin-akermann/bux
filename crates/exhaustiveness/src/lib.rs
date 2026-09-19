//! Exhaustiveness: every `match` of a module answers for every value it may meet.
//!
//! The phase consumes the typed tree and yields nothing, because it adds nothing: it either has
//! nothing to say, or it refuses the file. `docs/specs/exhaustiveness.md` is the specification,
//! and `docs/design.md` section 4 is the rule it enforces, that adding a variant points the
//! compiler at every match which has not been told what to do with it.

mod error;
mod pattern;
mod space;
mod usefulness;
mod walk;

use lumen_types::TypedProgram;

pub use crate::error::MatchError;

/// Checks that every `match` of `typed` covers the type it matches.
///
/// # Errors
///
/// Returns the first `match` that leaves a value of that type unanswered.
pub fn check(typed: &TypedProgram) -> Result<(), MatchError> {
    let resolved = typed.resolved();
    let program = resolved.program();
    let space = space::Space::of(program);
    let reading = pattern::Reading::new(resolved, &space);
    walk::module(program, &reading, &space)
}
