//! Expressions: their precedence, and the shapes a value is written in.

use crate::common::inner;

/// Wraps `expression` in the smallest function that can hold it.
fn in_function(expression: &str) -> Vec<String> {
    let source = format!("fn f() {{\n    {expression}\n}}");
    inner(&source, 2, 2)
}

#[test]
fn a_literal_carries_the_value_it_spells() {
    assert_eq!(in_function("42"), ["integer 42"]);
    assert_eq!(in_function("true"), ["bool true"]);
    assert_eq!(in_function("false"), ["bool false"]);
    assert_eq!(in_function("()"), ["unit"]);
}

#[test]
fn a_minus_straight_before_the_digits_is_part_of_the_number() {
    assert_eq!(in_function("-7"), ["integer -7"]);
    assert_eq!(
        in_function("-9223372036854775808"),
        ["integer -9223372036854775808"]
    );
    assert_eq!(in_function("-a"), ["unary Negate", "  name a"]);
}

#[test]
fn a_string_literal_is_decoded() {
    assert_eq!(in_function(r#""hello""#), [r#"string "hello""#]);
    assert_eq!(in_function(r#""a\"b""#), [r#"string "a\"b""#]);
    assert_eq!(in_function(r#""a\nb""#), [r#"string "a\nb""#]);
    assert_eq!(in_function(r#""a\\b""#), [r#"string "a\\b""#]);
}

#[test]
fn a_tighter_operator_sits_below_a_looser_one() {
    assert_eq!(
        in_function("a + b * c"),
        [
            "binary Add",
            "  name a",
            "  binary Multiply",
            "    name b",
            "    name c",
        ]
    );
}

/// Each expression beside the operator precedence leaves at its root.
const ROOTS: [(&str, &str); 8] = [
    ("a - b - c", "binary Subtract"),
    ("a * b / c", "binary Divide"),
    ("a + b * c", "binary Add"),
    ("a == b + c", "binary Equal"),
    ("a && b == c", "binary And"),
    ("a || b && c", "binary Or"),
    ("!a && b", "binary And"),
    ("(a + b) * c", "binary Multiply"),
];

#[test]
fn the_loosest_operator_of_an_expression_ends_up_at_its_root() {
    for (source, root) in ROOTS {
        assert_eq!(in_function(source)[0], root, "{source}");
    }
}

#[test]
fn a_repeated_operator_groups_to_the_left() {
    assert_eq!(in_function("a - b - c"), in_function("(a - b) - c"));
    assert_eq!(in_function("a / b / c"), in_function("(a / b) / c"));
}

#[test]
fn a_prefix_operator_binds_tighter_than_any_infix_one() {
    assert_eq!(
        in_function("!a && b"),
        ["binary And", "  unary Not", "    name a", "  name b"]
    );
    assert_eq!(
        in_function("-a + b"),
        ["binary Add", "  unary Negate", "    name a", "  name b"]
    );
}

#[test]
fn a_call_a_field_and_a_try_chain_left_to_right() {
    assert_eq!(
        in_function("users.first().name?"),
        [
            "try",
            "  field name",
            "    call",
            "      field first",
            "        name users",
        ]
    );
}

#[test]
fn a_call_carries_its_arguments_in_order() {
    assert_eq!(
        in_function("save(user, 1)"),
        ["call", "  name save", "  name user", "  integer 1"]
    );
}

#[test]
fn a_record_literal_carries_its_fields_in_order() {
    assert_eq!(
        in_function(r#"User { id: id, name: "Alice" }"#),
        [
            "record User",
            "  field-value id",
            "    name id",
            "  field-value name",
            r#"    string "Alice""#,
        ]
    );
}

#[test]
fn a_record_update_parses_as_the_same_node_as_a_record_literal() {
    assert_eq!(
        in_function(r#"user { name: "Bob" }"#),
        ["record user", "  field-value name", r#"    string "Bob""#]
    );
}

#[test]
fn a_record_literal_may_span_lines() {
    assert_eq!(
        in_function("User {\n        id: id,\n        active: true\n    }"),
        in_function("User { id: id, active: true }")
    );
}

#[test]
fn an_expression_continues_across_a_line_break_after_an_operator() {
    assert_eq!(
        in_function("a +\n        b"),
        ["binary Add", "  name a", "  name b"]
    );
}

#[test]
fn an_if_is_an_expression_with_a_condition_and_a_block() {
    assert_eq!(
        in_function("if a {\n        b\n    }"),
        ["if", "  branch", "    name a", "    block", "      name b"]
    );
}

#[test]
fn an_else_takes_a_block_or_another_if() {
    assert_eq!(
        in_function("if a {\n        b\n    } else {\n        c\n    }"),
        [
            "if",
            "  branch",
            "    name a",
            "    block",
            "      name b",
            "  else",
            "    block",
            "      name c",
        ]
    );
    assert_eq!(
        in_function("if a {\n    } else if b {\n    }"),
        [
            "if",
            "  branch",
            "    name a",
            "    block",
            "  branch",
            "    name b",
            "    block",
        ]
    );
}

#[test]
fn a_record_literal_needs_parentheses_where_a_block_would_follow() {
    assert_eq!(
        in_function("if user {\n    }"),
        ["if", "  branch", "    name user", "    block"]
    );
    assert_eq!(
        in_function("if (user { active: true }).active {\n    }"),
        [
            "if",
            "  branch",
            "    field active",
            "      record user",
            "        field-value active",
            "          bool true",
            "    block",
        ]
    );
}
