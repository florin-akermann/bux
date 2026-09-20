//! Patterns, as written in the arms of a `match`.

use crate::common::inner;

/// Wraps `arms` in the smallest `match` that can hold them.
fn in_match(arms: &str) -> Vec<String> {
    let source = format!("fn f() {{\n    match payment {{\n{arms}\n    }}\n}}");
    inner(&source, 3, 3)
}

#[test]
fn a_match_carries_its_scrutinee_and_its_arms_in_order() {
    assert_eq!(
        in_match("        Pending => 1\n        Captured => 2"),
        [
            "name payment",
            "arm",
            "  pattern Pending",
            "  integer 1",
            "arm",
            "  pattern Captured",
            "  integer 2",
        ]
    );
}

#[test]
fn a_variant_may_be_matched_on_what_it_carries_positionally() {
    assert_eq!(
        in_match("        Failed(reason) => reason"),
        [
            "name payment",
            "arm",
            "  pattern-tuple Failed",
            "    pattern reason",
            "  name reason",
        ]
    );
}

#[test]
fn a_variant_may_be_matched_on_the_fields_it_names() {
    assert_eq!(
        in_match("        Authorized { id, at } => id"),
        [
            "name payment",
            "arm",
            "  pattern-record Authorized",
            "    pattern-field id",
            "    pattern-field at",
            "  name id",
        ]
    );
}

#[test]
fn a_literal_may_be_matched_on() {
    assert_eq!(
        in_match("        1 => a\n        \"x\" => b\n        true => c"),
        [
            "name payment",
            "arm",
            "  pattern-integer 1",
            "  name a",
            "arm",
            r#"  pattern-string "x""#,
            "  name b",
            "arm",
            "  pattern-bool true",
            "  name c",
        ]
    );
}

#[test]
fn an_arm_body_may_sit_on_the_line_below_its_pattern() {
    assert_eq!(
        in_match("        Pending =>\n            1\n\n        Captured =>\n            2"),
        in_match("        Pending => 1\n        Captured => 2")
    );
}

#[test]
fn a_nested_pattern_matches_what_a_variant_carries() {
    assert_eq!(
        in_match("        Failed(Reason(text)) => text"),
        [
            "name payment",
            "arm",
            "  pattern-tuple Failed",
            "    pattern-tuple Reason",
            "      pattern text",
            "  name text",
        ]
    );
}

#[test]
fn a_variant_of_another_module_is_matched_through_the_name_it_is_reached_by() {
    assert_eq!(
        in_match("        demo.Pending => 1"),
        [
            "name payment",
            "arm",
            "  pattern demo.Pending",
            "  integer 1"
        ]
    );
    assert_eq!(
        in_match("        demo.Failed(reason) => reason"),
        [
            "name payment",
            "arm",
            "  pattern-tuple demo.Failed",
            "    pattern reason",
            "  name reason",
        ]
    );
    assert_eq!(
        in_match("        demo.Authorized { id } => id"),
        [
            "name payment",
            "arm",
            "  pattern-record demo.Authorized",
            "    pattern-field id",
            "  name id",
        ]
    );
}

#[test]
fn an_underscore_is_a_pattern_of_its_own_and_binds_no_name() {
    assert_eq!(
        in_match("        _ => 1"),
        ["name payment", "arm", "  pattern-wildcard", "  integer 1"]
    );
}

#[test]
fn alternatives_written_with_a_pipe_between_them_are_one_pattern() {
    assert_eq!(
        in_match("        Pending | Captured => 1"),
        any_of(&["Pending", "Captured"])
    );
}

#[test]
fn three_alternatives_are_one_pattern_and_not_two_nested_ones() {
    assert_eq!(
        in_match("        Pending | Captured | Settled => 1"),
        any_of(&["Pending", "Captured", "Settled"])
    );
}

/// The nodes one arm becomes whose pattern writes `alternatives` with a `|` between them.
fn any_of(alternatives: &[&str]) -> Vec<String> {
    let mut nodes = vec![
        "name payment".to_owned(),
        "arm".to_owned(),
        "  pattern-or".to_owned(),
    ];
    for name in alternatives {
        nodes.push(format!("    pattern {name}"));
    }
    nodes.push("  integer 1".to_owned());
    nodes
}

#[test]
fn an_alternative_is_a_whole_pattern_so_one_is_written_inside_a_constructor() {
    assert_eq!(
        in_match("        Failed(0 | 1) => 1"),
        [
            "name payment",
            "arm",
            "  pattern-tuple Failed",
            "    pattern-or",
            "      pattern-integer 0",
            "      pattern-integer 1",
            "  integer 1",
        ]
    );
}

#[test]
fn a_pipe_with_no_pattern_after_it_is_refused_where_the_pattern_belongs() {
    let refused =
        lumen_parser::parse("fn f() {\n    match payment {\n        Pending | => 1\n    }\n}\n")
            .expect_err("a pattern is expected after the pipe");

    assert_eq!(refused.message(), "expected a pattern, found `=>`");
}
