//! `docs/specs/naming.md`: a function whose result is `Bool` reads as a predicate.

use crate::common::{inferred, refusal};

/// A module whose one function is named `named` and gives back `gives`.
///
/// The body is `todo`, which takes whatever type is expected of it, so every module here differs
/// from the next in its name and its result alone, which is what the rule is about.
fn declaring(named: &str, gives: &str) -> String {
    format!(
        "fn {named}(user: User) -> {gives} {{\n    todo(\"the body is not the point\")\n}}\n\n\
             type User = {{\n    active: Bool\n}}\n"
    )
}

#[test]
fn a_bool_function_named_for_a_command_says_what_its_name_should_do() {
    let error = refusal(&declaring("active", "Bool"));

    assert_eq!(
        error.message(),
        "`active` gives back a `Bool`, so its name asks the question it answers"
    );
}

#[test]
fn the_help_names_the_four_prefixes_a_question_begins_with() {
    let error = refusal(&declaring("active", "Bool"));

    assert_eq!(
        error.help(),
        "begin the name with `is_`, `has_`, `can_`, or `should_`"
    );
}

#[test]
fn a_name_that_asks_its_question_compiles() {
    for asking in ["is_active", "has_paid", "can_edit", "should_retry"] {
        inferred(&declaring(asking, "Bool"));
    }
}

#[test]
fn a_function_that_gives_back_anything_else_is_named_however_it_reads_best() {
    inferred(&declaring("active", "Int"));
}

#[test]
fn a_function_that_writes_no_result_type_is_held_to_the_one_it_turned_out_to_have() {
    let source = "fn active(count: Int) {\n    count > 1\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`active` gives back a `Bool`, so its name asks the question it answers"
    );
}

#[test]
fn the_refusal_points_at_the_name_because_the_name_is_what_changes() {
    let source = declaring("active", "Bool");

    let error = refusal(&source);

    assert_eq!(error.span().text(&source), "active");
}

#[test]
fn a_prefix_that_only_opens_a_longer_word_is_not_the_question() {
    let error = refusal(&declaring("island", "Bool"));

    assert_eq!(
        error.message(),
        "`island` gives back a `Bool`, so its name asks the question it answers"
    );
}

#[test]
fn a_prefix_written_inside_the_name_is_not_where_the_question_is_asked() {
    let error = refusal(&declaring("count_is_active", "Bool"));

    assert_eq!(
        error.message(),
        "`count_is_active` gives back a `Bool`, so its name asks the question it answers"
    );
}
