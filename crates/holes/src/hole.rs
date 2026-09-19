//! One hole, and the refusal a build turns it into.

use lumen_ast::Span;
use lumen_diagnostics::{Code, Diagnostic};

/// A `todo(…)` written where a value belongs.
///
/// A hole carries only where it is written, because the reason is written there too: the
/// rendered line under the message is what names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hole {
    span: Span,
}

impl Hole {
    pub(crate) const fn at(span: Span) -> Self {
        Self { span }
    }

    /// The source this hole points at, which is the whole of the `todo(…)`.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// This hole as the diagnostic a build refuses it with.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new(
            Code::HoleBuilt,
            Self::message().to_owned(),
            self.span,
            Some(Self::help().to_owned()),
        )
    }

    /// The `error:` line, without its prefix.
    const fn message() -> &'static str {
        "this hole is not compiled"
    }

    /// The `help:` line, which says where a hole is welcome instead.
    const fn help() -> &'static str {
        "`lumen check` accepts a hole; fill it in, or build a module that holds none"
    }
}
