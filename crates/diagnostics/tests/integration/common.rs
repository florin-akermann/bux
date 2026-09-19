//! Helpers shared by the diagnostics tests.

use lumen_diagnostics::{Code, Diagnostic, render};
use lumen_lexer::Span;

/// The file name every rendering in these tests points at.
pub const FILE: &str = "demo.lm";

/// A diagnostic about the first occurrence of `about` in `source`, rendered against it.
///
/// Naming the text rather than the offsets is what lets a test read as the layout it is about.
pub fn rendered(source: &str, about: &str, help: Option<&str>) -> String {
    let start = source
        .find(about)
        .expect("the source says what it is about");
    render(&chained(Span::new(start, about.len()), help), source, FILE)
}

/// The one diagnostic these tests are written about, pointing at `span`.
pub fn chained(span: Span, help: Option<&str>) -> Diagnostic {
    Diagnostic::new(
        Code::ChainedComparison,
        "comparisons do not chain".to_owned(),
        span,
        help.map(str::to_owned),
    )
}
