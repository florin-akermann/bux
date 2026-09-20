//! The examples a module states about its functions, and the module that runs them.
//!
//! `docs/specs/doc-examples.md` is the specification, and `docs/design.md` section 11 is the rule
//! it serves: a signature says what a function takes and gives back, and an example says what it
//! does. An example is a line of the comment above a function, so nothing here reads the tree
//! alone: comments are not part of it, and the text they were written in is where they live.
//!
//! Reading them is one job and running them is another. [`stated_by`] reads, and holds the module
//! to the rule that every function states one, which is what `lumen build` refuses. [`Run`] writes
//! the module that tries them, which is what `lumen test` runs.

mod example;
mod read;
mod refusal;
mod run;

use lumen_ast::Program;

pub use crate::example::Example;
pub use crate::refusal::Refusal;
pub use crate::run::Run;

/// The examples `program` states, in the order they are written.
///
/// # Errors
///
/// Returns every function that states none, and every example written where nothing carries one,
/// in the order they are written. A build is how a reader learns what is left, so the first is
/// not the whole answer.
pub fn stated_by(source: &str, program: &Program) -> Result<Vec<Example>, Vec<Refusal>> {
    read::module(source, program)
}
