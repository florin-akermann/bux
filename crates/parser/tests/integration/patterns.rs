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
