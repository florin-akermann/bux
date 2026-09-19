//! The one error the check can fail with.

use lumen_ast::Span;
use lumen_diagnostics::{Code, Diagnostic};

use crate::pattern::Pat;

/// A `match` that leaves a value unanswered, and where it is written.
///
/// The check stops at the first one, so there is exactly one of these per failed run. The wording
/// is specified in `docs/specs/exhaustiveness.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchError {
    uncovered: Vec<String>,
    span: Span,
}

impl MatchError {
    /// The source the error points at, which is the whole `match`.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// This failure as the diagnostic the reader is shown.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new(
            Code::NonExhaustiveMatch,
            self.message(),
            self.span,
            Some(Self::help().to_owned()),
        )
    }

    /// The `error:` line, without its prefix.
    #[must_use]
    pub fn message(&self) -> String {
        let named: Vec<String> = self
            .uncovered
            .iter()
            .map(|pattern| format!("`{pattern}`"))
            .collect();
        format!("this `match` does not cover {}", named.join(", "))
    }

    /// The `help:` line, which every one of these has.
    #[must_use]
    pub const fn help() -> &'static str {
        "every value has an arm, or a name that binds whatever the arms before it did not"
    }

    /// The failure of the `match` at `span`, which leaves each of `uncovered` unanswered.
    pub(crate) fn at(span: Span, uncovered: &[Vec<Pat>]) -> Self {
        Self {
            uncovered: uncovered
                .iter()
                .filter_map(|row| row.first())
                .map(ToString::to_string)
                .collect(),
            span,
        }
    }
}
