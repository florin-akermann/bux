//! Statements: bindings, assignment, loops, and the ways a function returns.

use crate::common::inner;

/// Wraps `statements` in the smallest function that can hold them.
fn in_function(statements: &str) -> Vec<String> {
    let source = format!("fn f() {{\n{statements}\n}}");
    inner(&source, 2, 2)
}

#[test]
fn a_walrus_binds_a_name_that_is_never_assigned_to_again() {
    assert_eq!(
        in_function("    total := 0"),
        ["binding Immutable total", "  integer 0"]
    );
}

#[test]
fn var_binds_a_name_that_may_be_assigned_to() {
    assert_eq!(
        in_function("    var total = 0"),
        ["binding Mutable total", "  integer 0"]
    );
}

#[test]
fn an_assignment_replaces_or_adds_to_what_a_name_holds() {
    assert_eq!(
        in_function("    total = 1"),
        ["assign Set", "  name total", "  integer 1"]
    );
    assert_eq!(
        in_function("    total += 1"),
        ["assign Add", "  name total", "  integer 1"]
    );
}

#[test]
fn a_field_may_be_the_target_of_an_assignment() {
    assert_eq!(
        in_function("    user.name = 1"),
        ["assign Set", "  field name", "    name user", "  integer 1"]
    );
}

#[test]
fn return_carries_a_value_or_nothing() {
    assert_eq!(in_function("    return 1"), ["return", "  integer 1"]);
    assert_eq!(in_function("    return"), ["return"]);
}

#[test]
fn break_and_continue_are_statements_on_their_own() {
    assert_eq!(in_function("    break"), ["break"]);
    assert_eq!(in_function("    continue"), ["continue"]);
}

#[test]
fn a_for_loops_over_a_collection_a_condition_or_nothing() {
    assert_eq!(
        in_function("    for user in users {\n    }"),
        ["for-in user", "  name users", "  block"]
    );
    assert_eq!(
        in_function("    for total < limit {\n    }"),
        [
            "for-while",
            "  binary Less",
            "    name total",
            "    name limit",
            "  block",
        ]
    );
    assert_eq!(in_function("    for {\n    }"), ["for", "  block"]);
}

#[test]
fn statements_are_separated_by_newlines_and_kept_in_order() {
    assert_eq!(
        in_function("    a := 1\n\n    b := 2"),
        [
            "binding Immutable a",
            "  integer 1",
            "binding Immutable b",
            "  integer 2",
        ]
    );
}

#[test]
fn a_block_may_hold_no_statements_at_all() {
    assert_eq!(in_function(""), [] as [String; 0]);
}

#[test]
fn a_loop_body_holds_the_statements_that_drive_it() {
    assert_eq!(
        in_function("    for user in users {\n        total += 1\n        break\n    }"),
        [
            "for-in user",
            "  name users",
            "  block",
            "    assign Add",
            "      name total",
            "      integer 1",
            "    break",
        ]
    );
}
