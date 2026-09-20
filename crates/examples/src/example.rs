//! One example, and the refusal a run turns it into.

use lumen_ast::Span;
use lumen_diagnostics::{Code, Diagnostic};

/// One line of Lumen a function states about itself.
///
/// The expression has the type `Bool` and is `true` when it runs, which is the whole of what an
/// example claims. It is held as the text it was written as, because the module that runs it is
/// written as text too.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Example {
    of: String,
    expression: String,
    span: Span,
}

impl Example {
    pub(crate) fn stated(of: &str, expression: &str, span: Span) -> Self {
        Self {
            of: of.to_owned(),
            expression: expression.to_owned(),
            span,
        }
    }

    /// The function this documents.
    #[must_use]
    pub fn documents(&self) -> &str {
        &self.of
    }

    /// The expression it states, as it is written.
    #[must_use]
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// The line it is written on, in the file it was read out of.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// This example as the refusal a run turns it into when it did not hold.
    #[must_use]
    pub fn did_not_hold(&self) -> Diagnostic {
        Diagnostic::new(
            Code::ExampleDoesNotHold,
            format!("the example of `{}` does not hold", self.of),
            self.span,
            Some(
                "an example states what a function works out, and this one states what it does not"
                    .to_owned(),
            ),
        )
    }
}
