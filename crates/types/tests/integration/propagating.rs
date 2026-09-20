//! What `?` gives back, and which kind of function it may be written in.
//!
//! `docs/design.md` section 5 states the rule: `?` hands the `Err` of a `Result` or the `None`
//! of an `Option` back, and each lands in a function that gives back the same kind, so nothing
//! is converted. `docs/specs/arithmetic.md` works it through for a division.

use crate::common::{inferred_type, refusal};

/// A function that gives back a `Result`, which a `?` in these tests reaches for one to hand on.
const FIND: &str = "fn find(id: Int) -> Result<String, Int> {\n    Ok(\"found\")\n}\n";

/// `written` as the body of a function taking two whole numbers and giving back `gives`.
fn dividing(gives: &str, written: &str) -> String {
    format!("fn scaled(total: Int, count: Int) -> {gives} {{\n    {written}\n}}\n")
}

#[test]
fn a_question_mark_gives_the_ok_and_leaves_with_the_error() {
    let source = format!(
        "fn load(id: Int) -> Result<String, Int> {{\n    name := find(id)?\n    Ok(name)\n}}\n\n{FIND}"
    );

    assert_eq!(inferred_type(&source, "find(id)?", 1), "String");
}

#[test]
fn a_question_mark_on_an_option_gives_what_the_some_carries_and_leaves_with_the_none() {
    let source = dividing("Option<Int>", "Some((total / count)?)");

    assert_eq!(inferred_type(&source, "(total / count)?", 1), "Int");
}

#[test]
fn a_division_divides_through_a_question_mark_one_per_division() {
    let source = dividing("Option<Int>", "Some(((total / count)? / 2)? + 2 * 5)");

    assert_eq!(inferred_type(&source, "((total / count)? / 2)?", 1), "Int");
}

#[test]
fn a_question_mark_reads_the_kind_it_propagates_off_the_body_where_no_signature_says() {
    let source = "fn halved(total: Int, count: Int) {\n    Some((total / count)?)\n}\n";

    assert_eq!(inferred_type(source, "(total / count)?", 1), "Int");
}

#[test]
fn a_function_that_recurses_through_its_own_question_mark_waits_for_its_body_to_say() {
    let source = "fn walked(total: Int) {\n    held := walked(total)?\n    Some(held)\n}\n";

    assert_eq!(inferred_type(source, "walked(total)?", 1), "_");
}

#[test]
fn a_question_mark_on_an_option_in_a_function_that_gives_back_a_result_is_refused() {
    let source = dividing("Result<Int, String>", "Ok((total / count)?)");

    assert_eq!(
        refusal(&source).message(),
        "expected `Result<_, _>`, found `Option<Int>`"
    );
}

#[test]
fn a_question_mark_on_a_result_in_a_function_that_gives_back_an_option_is_refused() {
    let source =
        format!("fn named(id: Int) -> Option<String> {{\n    Some(find(id)?)\n}}\n\n{FIND}");

    assert_eq!(
        refusal(&source).message(),
        "expected `Option<_>`, found `Result<String, Int>`"
    );
}

#[test]
fn a_question_mark_in_a_function_that_gives_back_neither_kind_is_refused() {
    let source = format!("fn go() -> Int {{\n    name := find(1)?\n    1\n}}\n\n{FIND}");

    assert_eq!(
        refusal(&source).message(),
        "expected `Int`, found `Result<_, Int>`"
    );
}
