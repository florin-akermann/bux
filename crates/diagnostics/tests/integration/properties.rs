//! The properties of `docs/specs/diagnostics.md`, checked on generated spans.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_diagnostics::{CODES, Diagnostic, render};
use lumen_lexer::Span;

use crate::common::FILE;

/// The lines a generated source is built from, including blank and non-ASCII ones.
const LINES: [&str; 6] = ["fn f() {", "    a < b", "", "    // é", "}", "\ttab"];

/// A source of zero or more of [`LINES`].
fn source(tc: &TestCase) -> String {
    let lines: Vec<&'static str> = tc.draw(gs::vecs(gs::sampled_from(&LINES)));
    lines.join("\n") + "\n"
}

/// A span anywhere in or past `source`, on a character boundary or not.
///
/// The renderer answers about any span at all, not only the ones a lexer produces, so the
/// generator does not confine itself to the ones that are well formed.
fn span(tc: &TestCase, source: &str) -> Span {
    let offsets: Vec<usize> = (0..=source.len() + 2).collect();
    Span::new(
        tc.draw(gs::sampled_from(&offsets)),
        tc.draw(gs::sampled_from(&offsets)),
    )
}

#[hegel::test]
fn a_rendering_names_a_line_and_a_column_inside_the_source(tc: TestCase) {
    let source = source(&tc);
    let code = tc.draw(gs::sampled_from(CODES));
    let rendered = render(
        &Diagnostic::new(
            code,
            "something is wrong".to_owned(),
            span(&tc, &source),
            None,
        ),
        &source,
        FILE,
    );
    let place = rendered.lines().nth(1).expect("a rendering names a place");
    let (line, column) = numbers_of(place);
    let named = source
        .lines()
        .nth(line - 1)
        .unwrap_or_else(|| panic!("{place} names a line of {source:?}"));
    assert!(
        column >= 1 && column <= named.chars().count() + 1,
        "{place} of {named:?}"
    );
}

#[hegel::test]
fn a_rendering_opens_with_the_code_and_ends_with_a_newline(tc: TestCase) {
    let source = source(&tc);
    let code = tc.draw(gs::sampled_from(CODES));
    let rendered = render(
        &Diagnostic::new(
            code,
            "something is wrong".to_owned(),
            span(&tc, &source),
            None,
        ),
        &source,
        FILE,
    );
    assert!(rendered.starts_with(&format!("error[{}]: ", code.number())));
    assert!(rendered.ends_with('\n'));
}

/// The line and the column of a `  --> file:line:column` line.
fn numbers_of(place: &str) -> (usize, usize) {
    let mut parts = place.rsplit(':');
    let column = parts.next().expect("a place names a column");
    let line = parts.next().expect("a place names a line");
    (
        line.parse().expect("a line is a number"),
        column.parse().expect("a column is a number"),
    )
}
