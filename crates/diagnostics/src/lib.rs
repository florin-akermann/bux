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

    /// Which kind of refusal this is, which is the stable handle a spec or a test cites.
    #[must_use]
    pub const fn code(&self) -> Code {
        self.code
    }

    /// The `error:` line, without its prefix.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Where this points, which is the source the message is about.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// The `help:` line, without its prefix, where there is a fix worth naming.
    #[must_use]
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    /// The same diagnostic, said about `span` instead.
    ///
    /// A command that compiles source it wrote itself is refused about what it wrote, and this
    /// is how such a refusal is put back where the author can read it.
    ///
    /// The edit is dropped, because an edit is a span of the source it was worked out against
    /// and that is the source this is being moved away from.
    #[must_use]
    pub fn about(self, span: Span) -> Self {
        Self {
            span,
            fix: None,
            ..self
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
