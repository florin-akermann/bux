//! What a module states about its examples that a build will not take.

use lumen_ast::Span;
use lumen_diagnostics::{Code, Diagnostic};

/// One thing a module states about its examples that `docs/specs/doc-examples.md` refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    why: Why,
    span: Span,
}

/// What a module did about its examples that a build will not take.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Why {
    /// A function states none, held as the name it is declared under.
    StatesNone(String),
    /// An example is written where nothing carries one.
    DocumentsNothing,
    /// A module declares the name a run reaches for, held as that name.
    ReachedForByTheRun(String),
}

impl Refusal {
    pub(crate) fn states_none(named: &str, span: Span) -> Self {
        Self {
            why: Why::StatesNone(named.to_owned()),
            span,
        }
    }

    pub(crate) const fn documents_nothing(span: Span) -> Self {
        Self {
            why: Why::DocumentsNothing,
            span,
        }
    }

    pub(crate) fn reaches_for(named: &str, span: Span) -> Self {
        Self {
            why: Why::ReachedForByTheRun(named.to_owned()),
            span,
        }
    }

    /// The source this points at: the name of the function, or the line that documents nothing.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// This as the diagnostic a build refuses it with.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic {
        let (code, message, help) = match &self.why {
            Why::StatesNone(named) => (
                Code::NoExample,
                format!("`{named}` states no example"),
                "write `// example: <an expression that is true>` in the comment above it",
            ),
            Why::DocumentsNothing => (
                Code::ExampleDocumentsNothing,
                "this example documents nothing".to_owned(),
                "an example goes in the comment above the function it documents",
            ),
            Why::ReachedForByTheRun(named) => (
                Code::ExampleRunReachesTheName,
                format!("a run of the examples reaches `{named}`, and this module declares it"),
                "call the declaration something else, or run the module rather than its examples",
            ),
        };
        Diagnostic::new(code, message, self.span, Some(help.to_owned()))
    }
}
