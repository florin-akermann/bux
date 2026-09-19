//! The properties of `docs/specs/diagnostics.md`, checked on generated spans.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_diagnostics::{CODES, Diagnostic, json, render};
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

/// The pieces a generated message is woven from, holding everything JSON cannot write plainly.
const AWKWARD: [&str; 8] = ["\"", "\\", "\n", "\r", "\t", "\u{1}", "é", "a < b"];

/// Generated text with awkward pieces woven through it, which is what a message quoting a source
/// can hold.
///
/// The pieces are interleaved rather than appended, so an awkward character at the very start of
/// the text, and an escape between two plain ones, are both shapes the generator reaches. One
/// piece is always drawn, so no run of the property is about plain text alone.
fn message(tc: &TestCase) -> String {
    let mut woven = String::new();
    for piece in tc.draw(gs::vecs(gs::sampled_from(&AWKWARD))) {
        woven.push_str(&tc.draw(gs::text()));
        woven.push_str(piece);
    }
    woven.push_str(tc.draw(gs::sampled_from(&AWKWARD)));
    woven.push_str(&tc.draw(gs::text()));
    woven
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

#[hegel::test]
fn a_data_form_is_one_line_whatever_text_the_diagnostic_holds(tc: TestCase) {
    let message = message(&tc);
    let code = tc.draw(gs::sampled_from(CODES));
    let held = Diagnostic::new(
        code,
        message.clone(),
        Span::new(0, 1),
        Some(message.clone()),
    );

    let written = json(&held, FILE);

    assert_eq!(
        written.matches('\n').count(),
        1,
        "{message:?} wrote {written:?}"
    );
    assert!(written.ends_with("}\n"), "{written:?}");
}

#[hegel::test]
fn text_a_data_form_writes_reads_back_as_the_text_it_was_given(tc: TestCase) {
    let message = message(&tc);
    let code = tc.draw(gs::sampled_from(CODES));

    let written = json(
        &Diagnostic::new(code, message.clone(), Span::new(0, 1), None),
        FILE,
    );

    assert_eq!(read_back(&written, "message"), message, "{written:?}");
}

/// The text of the field `named`, with every escape the data form wrote turned back.
///
/// This is the reader a tool would have, written out so that the property is a round trip and
/// not a second copy of the escaping it is about.
fn read_back(written: &str, named: &str) -> String {
    let opens = written
        .find(&format!("\"{named}\":\""))
        .expect("the field is there")
        + named.len()
        + 4;
    let mut read = String::new();
    let mut characters = written[opens..].chars();
    while let Some(character) = characters.next() {
        match character {
            '"' => return read,
            '\\' => read.push(unescaped(&mut characters)),
            plain => read.push(plain),
        }
    }
    panic!("{written:?} never closes {named}")
}

/// What one escape stood for, read from just after its backslash.
fn unescaped(characters: &mut std::str::Chars<'_>) -> char {
    match characters.next().expect("an escape is not the last thing") {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        'u' => {
            let digits: String = characters.by_ref().take(4).collect();
            let number = u32::from_str_radix(&digits, 16).expect("four hex digits");
            char::from_u32(number).expect("a character")
        }
        itself => itself,
    }
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
