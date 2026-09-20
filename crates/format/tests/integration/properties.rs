//! The properties of `docs/specs/formatting.md`, checked on generated programs.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_diagnostics::render;
use lumen_format::{CheckError, check, format};

use crate::common::tree;

/// The item shapes a generated program is built from, each already in canonical form.
const ITEMS: [&str; 8] = [
    "import io",
    "type UserId = UserId(Int)",
    "type User = {\n    id: UserId\n}",
    "type Payment =\n    | Pending\n    | Failed(String)",
    "fn added(count: Int) -> Int {\n    // why this is here\n    count + 1 * (2 - 3)\n}",
    "fn dropped(count: Int) -> Int {\n    _ = added(count)\n    count\n}",
    "fn named(count: Int) -> Int {\n    joined(first: count, second: count)\n}",
    "fn matched(payment: Payment) -> Int {\n    match payment {\n        Pending => 1\n        Failed(reason) => 2\n    }\n}",
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
    items.push(format!("fn held() {{\n    {}\n}}", expression(tc)));
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

#[hegel::test]
fn applying_the_fix_of_a_file_that_departs_from_canonical_form_makes_it_canonical(tc: TestCase) {
    let written = spelled_as_a_person_might(&tc);
    let Err(error @ CheckError::NotCanonical { .. }) = check(&written) else {
        panic!("{written:?} is spelled wrongly somewhere, so canonical form refuses it")
    };
    let diagnostic = error.diagnostic();
    let fix = diagnostic
        .fix()
        .expect("a file that departs from canonical form carries the edit that ends the departure");

    let fixed = fix.applied_to(&written);

    assert_eq!(format(&fixed).as_deref(), Ok(fixed.as_str()), "{written:?}");
}

/// A generated program written the way a person writes one, which is never canonical form.
///
/// Blank lines and indentation are what a person gets wrong, so they are what the generator gets
/// wrong, at any line break and not only the first: a run of blank lines deep in a file is the
/// case one edit at a time would not answer. One break is always spelled wrongly, so the program
/// that comes back always departs from canonical form and the property is never vacuous.
fn spelled_as_a_person_might(tc: &TestCase) -> String {
    let program = program(tc);
    let lines: Vec<&str> = program.split('\n').collect();
    let breaks: Vec<usize> = (1..lines.len()).collect();
    let always_wrong = tc.draw(gs::sampled_from(&breaks));
    let mut written = String::with_capacity(program.len());
    for (index, line) in lines.iter().enumerate() {
        if index == always_wrong {
            written.push_str(tc.draw(gs::sampled_from(&SPACING[1..])));
        } else if index > 0 {
            written.push_str(tc.draw(gs::sampled_from(&SPACING)));
        }
        written.push_str(line);
    }
    written
}

/// What a person writes where canonical form writes one line break, the first being the right one.
const SPACING: [&str; 6] = ["\n", "\n\n", "\n\n\n", "\n   \n", "  \n", "\n\t"];

/// The comment lines of a text, in order and without their indentation.
fn comment_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("//"))
        .collect()
}

/// What a generated name opens with, including the underscore a person leads one with.
///
/// An underscore never stands alone, because `_` is a keyword rather than a name.
const OPENERS: [&str; 6] = ["a", "A", "u", "U", "_a", "_U"];

/// The characters the rest of a generated name is drawn from, which are every kind it may hold.
const LETTERS: &str = "abUI_2";

/// How long the rest of a generated name runs, past the opener that is always written.
const LONGEST: usize = 5;

/// A name a person might write, in any mixture of case, underscore, and digit.
fn a_name(tc: &TestCase) -> String {
    let opener = tc.draw(gs::sampled_from(&OPENERS));
    let rest: String = tc.draw(gs::text().alphabet(LETTERS).max_size(LONGEST));
    format!("{opener}{rest}")
}

/// A module declaring one function called `name`, which is the only thing about it that varies.
fn declaring(name: &str) -> String {
    format!("fn {name}(count: Int) -> Int {{\n    count\n}}\n")
}

/// The spelling canonical form advises for `name`, where it advises one.
///
/// A name already canonical is its own spelling. A name that is an initial has none, because the
/// word it should have named is the author's to choose and no rewrite can guess it.
fn spelling(name: &str) -> Option<String> {
    let Err(error) = check(&declaring(name)) else {
        return Some(name.to_owned());
    };
    render(&error.diagnostic(), &declaring(name), "demo.lm")
        .lines()
        .find_map(|line| line.strip_prefix("help: canonical form spells this name `"))
        .and_then(|spelled| spelled.strip_suffix('`'))
        .map(str::to_owned)
}

#[hegel::test]
fn every_spelling_canonical_form_advises_is_one_it_accepts(tc: TestCase) {
    let written = a_name(&tc);

    let Some(spelled) = spelling(&written) else {
        return;
    };

    assert_eq!(
        check(&declaring(&spelled)),
        Ok(()),
        "{written} -> {spelled}"
    );
}
