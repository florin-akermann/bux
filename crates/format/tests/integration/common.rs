//! Helpers shared by the formatter's behaviour tests.
//!
//! A formatting behaviour is stated by writing a source and the canonical text beside it, so
//! [`formatted`] is what nearly every test calls. [`tree`] is what states the other half: that
//! formatting never changes what a program says.

use lumen_format::format;
use lumen_parser::parse;

/// The canonical text of `source`.
pub fn formatted(source: &str) -> String {
    format(source).unwrap_or_else(|error| panic!("{source:?} parses: {}", error.message()))
}

/// The canonical text of `body` inside the smallest function that can hold it.
///
/// The wrapper's own two lines are dropped, and the one level of indentation with them, so a
/// test about a statement reads as that statement.
pub fn in_function(body: &str) -> Vec<String> {
    let formatted = formatted(&format!("fn f() {{\n{body}\n}}\n"));
    formatted
        .lines()
        .skip(1)
        .take_while(|line| *line != "}")
        .map(|line| line.strip_prefix("    ").unwrap_or(line).to_owned())
        .collect()
}

/// The parse tree of `source` as text, with every span erased.
///
/// Two sources say the same thing when these are equal. Spans are left out because formatting
/// moves every offset in the file, which is the one difference it is allowed to make.
pub fn tree(source: &str) -> String {
    let parsed =
        parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {}", error.message()));
    erase_spans(&format!("{parsed:?}"))
}

/// Replaces every `Span { … }` of a rendered tree with the bare word, leaving the shape.
fn erase_spans(rendered: &str) -> String {
    let mut erased = String::with_capacity(rendered.len());
    let mut rest = rendered;
    while let Some(start) = rest.find(SPAN) {
        erased.push_str(&rest[..start]);
        erased.push_str("Span");
        let past = rest[start..]
            .find('}')
            .expect("a rendered span closes its brace");
        rest = &rest[start + past + 1..];
    }
    erased.push_str(rest);
    erased
}

const SPAN: &str = "Span { ";
