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
    let diagnostic = Diagnostic::new(
        Code::ChainedComparison,
        "comparisons do not chain".to_owned(),
        Span::new(start, about.len()),
        help.map(str::to_owned),
    );
    render(&diagnostic, source, FILE)
}
