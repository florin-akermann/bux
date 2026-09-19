//! Holes: the unfinished bodies `lumen check` accepts and `lumen build` refuses.
//!
//! The phase consumes the typed tree and yields either the holes it holds or a [`Whole`], which
//! is what the phases after it take. It adds nothing to the tree, because a hole is well typed:
//! `docs/specs/holes.md` is the specification, and `docs/design.md` section 5 is the rule it
//! serves, that unfinished work says so in the language rather than in a comment.

mod hole;
mod walk;

use lumen_types::TypedProgram;

pub use crate::hole::Hole;

/// A typed module that holds no hole, which is the only kind there is anything to lower.
///
/// A hole has nothing to run, so lowering one would have nothing to write. Taking one of these
/// is how lowering says so: a module that holds a hole cannot be handed to it, rather than being
/// handed over and refused afterwards.
#[derive(Clone, Copy, Debug)]
pub struct Whole<'a>(&'a TypedProgram);

impl<'a> Whole<'a> {
    /// `typed` when it holds no hole at all.
    ///
    /// # Errors
    ///
    /// Returns every hole `typed` holds, in the order they are written, because a build is how
    /// a reader learns what is left and a list of one would not be that list.
    pub fn of_module(typed: &'a TypedProgram) -> Result<Self, Vec<Hole>> {
        match walk::module(typed.resolved()) {
            found if found.is_empty() => Ok(Self(typed)),
            found => Err(found),
        }
    }

    /// The module itself, which every phase after this one reads.
    #[must_use]
    pub const fn typed(&self) -> &TypedProgram {
        self.0
    }
}
