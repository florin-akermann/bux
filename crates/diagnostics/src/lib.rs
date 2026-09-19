//! What the compiler refuses with, and how a refusal reads.
//!
//! `docs/specs/diagnostics.md` states the layout, the codes, and the exit codes, and
//! `docs/implementation.md` section 8 sets the voice. A [`Code`] is the stable handle on one kind
//! of refusal, so a spec or a test cites the code and never the prose. [`render`] writes a
//! [`Diagnostic`] the way the reader sees it.
//!
//! No diagnostic mentions the JVM, a class file, or a stack frame.

mod code;
mod fix;
mod json;
mod render;

use lumen_lexer::Span;

pub use crate::code::{CODES, Code};
pub use crate::fix::Fix;
pub use crate::json::json;
pub use crate::render::render;

/// One thing the compiler refuses, in the words the reader sees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    code: Code,
    message: String,
    span: Span,
    help: Option<String>,
    /// The edit that answers it, where the compiler knows one; `docs/specs/diagnostics.md` says
    /// which refusals have one and why the rest do not.
    fix: Option<Fix>,
}

impl Diagnostic {
    /// What `code` says about `span`, and what to write instead where there is something to say.
    #[must_use]
    pub const fn new(code: Code, message: String, span: Span, help: Option<String>) -> Self {
        Self {
            code,
            message,
            span,
            help,
            fix: None,
        }
    }

    /// The same diagnostic, carrying the edit that answers it.
    #[must_use]
    pub fn fixed_by(self, fix: Fix) -> Self {
        Self {
            fix: Some(fix),
            ..self
        }
    }

    /// The edit that answers it, which most diagnostics do not have.
    #[must_use]
    pub const fn fix(&self) -> Option<&Fix> {
        self.fix.as_ref()
    }
}
