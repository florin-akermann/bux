//! `docs/specs/patterns.md`: what `_`, a literal, and an or-pattern each ask of what they match.

use crate::common::{inferred, inferred_type, refusal};

/// A declared type taking a whole number and comparing two of them, which a pattern needs both of.
const INT32: &str = concat!(
    "derive Eq for Int32\n\n",
    "instance IntegerLiteral<Int32> {\n",
    "    fn lowest() -> Int {\n        -2147483648\n    }\n\n",
    "    fn highest() -> Int {\n        2147483647\n    }\n\n",
    "    fn from_literal(literal: Int) -> Int32 {\n        Int32(literal)\n    }\n}\n\n",
    "type Int32 = Int32(Int)\n"
);

/// A module declaring the payment type, with `arms` as the arms of the one `match`.
fn matching(arms: &str) -> String {
    format!(
        "fn describe(payment: Payment) -> String {{\n    match payment {{\n{arms}    }}\n}}\n\n\
         type Payment =\n    | Pending\n    | Running\n    | Failed(String)\n"
    )
}

#[test]
fn an_underscore_asks_nothing_of_what_it_matches() {
    let source = matching("        _ => \"whatever\"\n");

    inferred(&source);
}

#[test]
fn every_alternative_of_an_or_pattern_is_held_to_the_type_that_is_matched() {
    let source = matching("        Pending | Running => \"waiting\"\n        _ => \"other\"\n");

    inferred(&source);
}

#[test]
fn an_alternative_of_another_type_is_refused_where_that_alternative_is_written() {
    let source = matching("        Pending | \"running\" => \"waiting\"\n");
    let refused = refusal(&source);

    assert_eq!(refused.message(), "expected `Payment`, found `String`");
    assert_eq!(refused.span().text(&source), "\"running\"");
}

#[test]
fn a_whole_number_pattern_takes_the_type_it_is_matched_against() {
    let source = format!(
        "fn counted(count: Int32) -> Int {{\n    match count {{\n        5 => 1\n        _ => 0\n    }}\n}}\n\n{INT32}"
    );

    assert_eq!(inferred_type(&source, "5", 1), "Int32");
}

#[test]
fn a_whole_number_pattern_over_a_type_with_no_eq_names_the_instance_that_is_missing() {
    let source = format!(
        "fn counted(count: Int32) -> Int {{\n    match count {{\n        5 => 1\n        _ => 0\n    }}\n}}\n\n{}",
        INT32.replace("derive Eq for Int32\n\n", "")
    );

    assert_eq!(
        refusal(&source).message(),
        "`Int32` has no instance of `Eq`"
    );
}

#[test]
fn a_whole_number_pattern_outside_the_range_its_instance_states_is_refused() {
    let source = format!(
        "fn counted(count: Int32) -> Int {{\n    match count {{\n        5000000000 => 1\n        _ => 0\n    }}\n}}\n\n{INT32}"
    );

    assert_eq!(
        refusal(&source).message(),
        "`5000000000` does not fit `Int32`, which holds `-2147483648` to `2147483647`"
    );
}

#[test]
fn a_string_pattern_is_a_string_and_takes_its_type_from_nowhere() {
    let source = format!(
        "fn counted(count: Int32) -> Int {{\n    match count {{\n        \"five\" => 1\n        _ => 0\n    }}\n}}\n\n{INT32}"
    );

    assert_eq!(
        refusal(&source).message(),
        "expected `Int32`, found `String`"
    );
}

#[test]
fn a_whole_number_pattern_over_a_type_that_takes_none_is_the_mismatch_it_always_was() {
    let source = "fn counted(open: Bool) -> Int {\n    match open {\n        1 => 1\n        _ => 0\n    }\n}\n";

    assert_eq!(refusal(source).message(), "expected `Bool`, found `Int`");
}

#[test]
fn an_alternative_written_inside_a_constructor_is_held_to_what_that_constructor_carries() {
    let source = matching("        Failed(0 | 1) => \"numbered\"\n        _ => \"other\"\n");

    assert_eq!(refusal(&source).message(), "expected `String`, found `Int`");
}
