//! The first thing that does not fit the grammar, and the words it is reported in.

use crate::common::render_error;

/// The `error:` line of the failure, without its span or its help.
fn message(source: &str) -> String {
    let rendered = render_error(source);
    let first = rendered.lines().next().expect("an error has a message");
    first
        .split_once(' ')
        .expect("a rendered error is a span and a message")
        .1
        .to_owned()
}

/// The `help:` line of the failure, when there is one.
fn help(source: &str) -> Option<String> {
    render_error(source)
        .lines()
        .nth(1)
        .map(|line| line.trim_start_matches("help: ").to_owned())
}

#[test]
fn a_function_without_a_name_is_a_parse_error() {
    assert_eq!(
        message("fn (x) {\n}"),
        "expected a function name, found `(`"
    );
    assert_eq!(
        help("fn (x) {\n}").as_deref(),
        Some("every function has a name; Lumen has no anonymous functions")
    );
}

#[test]
fn a_file_may_only_hold_an_import_a_type_or_a_function() {
    assert_eq!(
        message("total := 1"),
        "expected an import, a type, or a function, found `total`"
    );
}

#[test]
fn a_missing_closing_bracket_names_the_bracket() {
    assert_eq!(message("fn f( {\n}"), "expected a name, found `{`");
    assert_eq!(
        message("fn f() {\n    a := 1"),
        "expected `}`, found the end of the file"
    );
}

#[test]
fn a_second_item_on_one_line_is_a_parse_error() {
    assert_eq!(
        message("import io import files"),
        "expected the end of the line, found `import`"
    );
}

#[test]
fn an_unterminated_string_is_reported_where_it_opens() {
    assert_eq!(
        render_error("fn f() {\n    a := \"oh\n}"),
        concat!(
            "18..21 this string has no closing quote\n",
            "help: add a closing `\"` before the end of the line\n",
        )
    );
}

#[test]
fn a_character_the_language_has_no_use_for_is_reported_where_it_sits() {
    assert_eq!(
        render_error("fn f() {\n    a := @\n}"),
        "18..19 this character is not part of the language\n"
    );
}

#[test]
fn an_unknown_escape_names_the_character_after_the_backslash() {
    let source = "fn f() {\n    a := \"a\\qb\"\n}";
    assert_eq!(message(source), "`\\q` is not an escape");
    assert_eq!(
        help(source).as_deref(),
        Some("the escapes are `\\\"`, `\\\\`, `\\n`, `\\t`, and `\\r`")
    );
}

#[test]
fn a_number_too_large_for_a_whole_number_is_a_parse_error() {
    let source = "fn f() {\n    a := 9223372036854775808\n}";
    assert_eq!(
        message(source),
        "this number does not fit in a whole number"
    );
    assert_eq!(
        help(source).as_deref(),
        Some("the largest whole number is 9223372036854775807")
    );
}

#[test]
fn a_comparison_does_not_chain() {
    let source = "fn f() {\n    a < b < c\n}";
    assert_eq!(message(source), "comparisons do not chain");
    assert_eq!(
        help(source).as_deref(),
        Some("compare twice and join the two with `&&`")
    );
}

#[test]
fn a_mutable_binding_is_joined_by_an_equals_sign_rather_than_a_walrus() {
    assert_eq!(
        message("fn f() {\n    var total := 0\n}"),
        "expected `=`, found `:=`"
    );
}

#[test]
fn a_variant_that_carries_nothing_is_written_without_parentheses() {
    assert_eq!(message("type T = Failed()"), "expected a type, found `)`");
}

#[test]
fn an_error_points_at_the_token_that_failed() {
    assert_eq!(
        render_error("import 1"),
        "7..8 expected a name, found a number\n"
    );
}
