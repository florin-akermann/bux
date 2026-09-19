//! What the compiler refuses with, and how a refusal reads.
//!
//! `docs/specs/diagnostics.md` states the layout, the codes, and the exit codes, and
//! `docs/implementation.md` section 8 sets the voice. A [`Code`] is the stable handle on one kind
//! of refusal, so a spec or a test cites the code and never the prose. [`render`] writes a
//! [`Diagnostic`] the way the reader sees it.
//!
//! No diagnostic mentions the JVM, a class file, or a stack frame.

mod code;
mod render;

use lumen_lexer::Span;

pub use crate::code::{CODES, Code};
pub use crate::render::render;

/// One thing the compiler refuses, in the words the reader sees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    code: Code,
    message: String,
    span: Span,
    help: Option<String>,
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
        }
    }
}
