//! The properties of `docs/specs/formatting.md`, checked on generated programs.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_format::format;

use crate::common::tree;

/// The item shapes a generated program is built from, each already in canonical form.
const ITEMS: [&str; 6] = [
    "import io",
    "type UserId = UserId(Int64)",
    "type User = {\n    id: UserId\n}",
    "type Payment =\n    | Pending\n    | Failed(String)",
    "fn f(a: Int) -> Int {\n    // why this is here\n    a + 1 * (2 - 3)\n}",
    "fn g(p: Payment) -> Int {\n    match p {\n        Pending => 1\n        Failed(reason) => 2\n    }\n}",
];

/// The atoms a generated expression is built from, one of which is a bare number.
const ATOMS: [&str; 4] = ["a", "7", "0", "f(1, b)"];

/// The ways a generated expression wraps the one inside it, `_` standing for that one.
///
/// A `-` in front of a number is the pair that matters most: the grammar reads `-7` as one
/// number, so a printer that drops the parentheses of `-(7.abs())` writes a different program.
const WRAPPERS: [&str; 10] = [
    "-_", "- _", "!_", "(_)", "_.abs()", "_?", "_ + b", "b * _", "(_) == b", "g( _ )",
];

/// How many wrappers one generated expression carries at most, well inside the nesting budget.
const DEEPEST: usize = 12;

/// An expression built by wrapping an atom in layers, spelled as a person might rather than
/// canonically, so that the printer has parentheses and blanks to decide about.
fn expression(tc: &TestCase) -> String {
    let wrappers: Vec<&'static str> =
        tc.draw(gs::vecs(gs::sampled_from(&WRAPPERS)).max_size(DEEPEST));
    let mut written = tc.draw(gs::sampled_from(&ATOMS)).to_owned();
    for wrapper in wrappers {
        written = wrapper.replace('_', &written);
    }
    written
}

/// A program of zero or more of [`ITEMS`], and one function of a generated expression.
fn program(tc: &TestCase) -> String {
    let mut items: Vec<String> = tc
        .draw(gs::vecs(gs::sampled_from(&ITEMS)))
        .into_iter()
        .map(str::to_owned)
        .collect();
    items.push(format!("fn h() {{\n    {}\n}}", expression(tc)));
    items.join("\n\n") + "\n"
}

#[hegel::test]
fn formatting_is_deterministic_and_idempotent(tc: TestCase) {
    let source = program(&tc);
    let once = format(&source).expect("a generated program parses");
    assert_eq!(format(&source).expect("parses"), once);
    assert_eq!(format(&once).expect("canonical text parses"), once);
}

#[hegel::test]
fn formatting_preserves_the_tree(tc: TestCase) {
    let source = program(&tc);
    let canonical = format(&source).expect("a generated program parses");
    assert_eq!(tree(&canonical), tree(&source));
}

#[hegel::test]
fn every_comment_survives_in_source_order(tc: TestCase) {
    let source = program(&tc);
    let canonical = format(&source).expect("a generated program parses");
    assert_eq!(comment_lines(&canonical), comment_lines(&source));
}

/// The comment lines of a text, in order and without their indentation.
fn comment_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("//"))
        .collect()
}
