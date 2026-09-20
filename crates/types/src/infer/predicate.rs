//! How a function whose result is `Bool` is named, which is the question it answers.
//!
//! `docs/specs/naming.md` states the rule: `active(user)` reads as a command at every call site,
//! and `is_active(user)` reads as the question it is. The result read is the one inference
//! settled, so a function that writes no result type is held to whatever type it turned out to
//! have, exactly as the flag rule of `docs/specs/arguments.md` reads a signature.

use lumen_ast::Function;

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::Inference;
use crate::types::Type;

/// The four prefixes a name asks its question with, which are all the questions a program asks.
const QUESTIONS: [&str; 4] = ["is_", "has_", "can_", "should_"];

impl Inference<'_> {
    /// The name of one function, held to the question its result says it answers.
    ///
    /// # Errors
    ///
    /// Returns the function whose result is `Bool` and whose name asks nothing.
    pub(crate) fn settle_predicate(&mut self, function: &Function) -> Result<(), TypeError> {
        if self.table.solved(&self.result) != Type::boolean()
            || asks_a_question(&function.name.text)
        {
            return Ok(());
        }
        let kind = TypeErrorKind::NotAPredicate(function.name.text.clone());
        Err(TypeError::at(function.name.span, kind))
    }
}

/// Whether `name` begins with one of the four prefixes, which is how it asks its question.
fn asks_a_question(name: &str) -> bool {
    QUESTIONS.iter().any(|opener| name.starts_with(opener))
}
