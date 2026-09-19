//! The two ways the check refuses a `match`.

use lumen_ast::Span;
use lumen_diagnostics::{Code, Diagnostic};

use crate::order::OutOfOrder;
use crate::pattern::Pat;

/// A `match` the check refuses, and where it is written.
///
/// The check stops at the first one, so there is exactly one of these per failed run. The wording
/// is specified in `docs/specs/exhaustiveness.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchError {
    refusal: Refusal,
}

/// What is wrong with the `match`: what it leaves out, or where it puts what it has.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Refusal {
    /// Values no arm answers for, each named as an arm would write it, and the whole `match`.
    NotCovered { uncovered: Vec<String>, span: Span },
    /// An arm written above one the type declares above it.
    OutOfOrder(OutOfOrder),
}

impl MatchError {
    /// This failure as the diagnostic the reader is shown.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new(self.code(), self.message(), self.span(), Some(self.help()))
    }

    /// The source the error points at: the whole `match`, or the arm that is out of place.
    #[must_use]
    pub const fn span(&self) -> Span {
        match &self.refusal {
            Refusal::NotCovered { span, .. } => *span,
            Refusal::OutOfOrder(out) => out.span(),
        }
    }

    /// The `error:` line, without its prefix.
    #[must_use]
    pub fn message(&self) -> String {
        match &self.refusal {
            Refusal::NotCovered { uncovered, .. } => {
                let named: Vec<String> = uncovered
                    .iter()
                    .map(|pattern| format!("`{pattern}`"))
                    .collect();
                format!("this `match` does not cover {}", named.join(", "))
            }
            Refusal::OutOfOrder(out) => out.message(),
        }
    }

    /// The `help:` line, which says what to write instead.
    #[must_use]
    pub fn help(&self) -> String {
        match &self.refusal {
            Refusal::NotCovered { .. } => {
                "every value has an arm, or a name that binds whatever the arms before it did not"
                    .to_owned()
            }
            Refusal::OutOfOrder(out) => out.help(),
        }
    }

    const fn code(&self) -> Code {
        match &self.refusal {
            Refusal::NotCovered { .. } => Code::NonExhaustiveMatch,
            Refusal::OutOfOrder(_) => Code::ArmOutOfOrder,
        }
    }

    /// The `match` at `span`, which leaves each of `uncovered` unanswered.
    pub(crate) fn not_covered(span: Span, uncovered: &[Vec<Pat>]) -> Self {
        Self {
            refusal: Refusal::NotCovered {
                uncovered: uncovered
                    .iter()
                    .filter_map(|row| row.first())
                    .map(ToString::to_string)
                    .collect(),
                span,
            },
        }
    }

    /// The `match` whose arms are not in the order the type declares its variants.
    pub(crate) const fn out_of_order(out: OutOfOrder) -> Self {
        Self {
            refusal: Refusal::OutOfOrder(out),
        }
    }
}
