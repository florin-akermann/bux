//! Expressions: spacing, and the parentheses the printer puts back.

use crate::common::in_function;

/// A source expression beside the canonical text of the same expression.
const SPACING: [(&str, &str); 15] = [
    ("[ 1 , 2 ]", "[1, 2]"),
    ("[ ]", "[]"),
    ("[\n    1,\n    2\n]", "[1, 2]"),
    ("a+b", "a + b"),
    ("a&&b||c", "a && b || c"),
    ("!a", "!a"),
    ("-a", "-a"),
    ("a . b", "a.b"),
    ("f ( a , b )", "f(a, b)"),
    ("f()", "f()"),
    ("f ( a : b , c : d )", "f(a: b, c: d)"),
    ("f(a:b)", "f(a: b)"),
    ("a ?", "a?"),
    ("User{id:id}", "User { id: id }"),
    ("User{}", "User {}"),
];

#[test]
fn an_operator_is_spaced_and_a_bracket_is_not() {
    for (source, canonical) in SPACING {
        assert_eq!(in_function(source), [canonical], "{source}");
    }
}

/// A source expression beside the canonical text, where parentheses decide the shape.
const PARENTHESES: [(&str, &str); 8] = [
    ("(a + b) * c", "(a + b) * c"),
    ("a + (b * c)", "a + b * c"),
    ("a - (b - c)", "a - (b - c)"),
    ("(a - b) - c", "a - b - c"),
    ("!(a && b)", "!(a && b)"),
    ("(!a) && b", "!a && b"),
    ("(a + b).c", "(a + b).c"),
    ("-(a.b)", "-a.b"),
];

#[test]
fn a_parenthesis_is_written_where_the_shape_needs_it_and_nowhere_else() {
    for (source, canonical) in PARENTHESES {
        assert_eq!(in_function(source), [canonical], "{source}");
    }
}

#[test]
fn a_literal_is_written_back_as_it_was_decoded() {
    assert_eq!(in_function("42"), ["42"]);
    assert_eq!(in_function("-7"), ["-7"]);
    assert_eq!(in_function("- 7"), ["-7"]);
    assert_eq!(in_function("true"), ["true"]);
    assert_eq!(in_function("()"), ["()"]);
}

#[test]
fn a_string_is_written_with_its_escapes_back_in() {
    assert_eq!(in_function(r#""a\"b""#), [r#""a\"b""#]);
    assert_eq!(in_function(r#""a\nb""#), [r#""a\nb""#]);
    assert_eq!(in_function(r#""a\\b""#), [r#""a\\b""#]);
    assert_eq!(in_function(r#""a\tb""#), [r#""a\tb""#]);
    assert_eq!(in_function(r#""a\rb""#), [r#""a\rb""#]);
}

#[test]
fn a_call_and_a_record_literal_stay_on_one_line_however_long_they_run() {
    assert_eq!(in_function("save(\n    user,\n    1\n)"), ["save(user, 1)"]);
    assert_eq!(
        in_function("rename(\n    from: old,\n    to: new\n)"),
        ["rename(from: old, to: new)"]
    );
    assert_eq!(
        in_function("User {\n    id: id,\n    name: name\n}"),
        ["User { id: id, name: name }"]
    );
}

#[test]
fn a_written_list_keeps_a_record_literal_unparenthesised_in_a_for_header() {
    assert_eq!(
        in_function("for u in [User{id:1}] {\n    }"),
        ["for u in [User { id: 1 }] {", "}"]
    );
}
