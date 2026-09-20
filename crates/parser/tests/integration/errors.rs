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
fn a_derive_names_a_trait_before_for_and_a_type_after_it() {
    assert_eq!(message("derive for User"), "expected a name, found `for`");
    assert_eq!(message("derive Eq User"), "expected `for`, found `User`");
    assert_eq!(
        message("derive Eq, for User"),
        "expected a name, found `for`"
    );
}

#[test]
fn a_file_may_only_hold_an_import_a_type_a_trait_an_instance_a_function_or_an_extern() {
    assert_eq!(
        message("total := 1"),
        "expected an import, a type, a trait, an instance, a derive, a function, or an extern, \
         found `total`"
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
fn a_call_names_all_of_its_arguments_or_none_of_them() {
    let half_named = "fn f() {\n    rename(from: old, new)\n}";
    let half_positional = "fn f() {\n    rename(old, to: new)\n}";
    let refusal = "this call names some of its arguments and not others";

    assert_eq!(message(half_named), refusal);
    assert_eq!(message(half_positional), refusal);
    assert_eq!(
        help(half_named).as_deref(),
        Some("a call names all of its arguments or none of them")
    );
}

#[test]
fn a_variant_that_carries_nothing_is_written_without_parentheses() {
    assert_eq!(message("type T = Failed()"), "expected a type, found `)`");
}

#[test]
fn a_type_argument_list_holds_at_least_one_type() {
    assert_eq!(
        message("type T = { ids: List<> }"),
        "expected a type, found `>`"
    );
    assert_eq!(message("fn f<>() {\n}"), "expected a name, found `>`");
}

#[test]
fn a_record_pattern_names_at_least_one_field() {
    assert_eq!(
        message("fn f() {\n    match a {\n        P {} => 1\n    }\n}"),
        "expected a name, found `}`"
    );
}

#[test]
fn a_program_that_nests_deeper_than_the_parser_descends_is_an_error() {
    let deep = format!(
        "fn f() {{\n    {}a{}\n}}",
        "(".repeat(20_000),
        ")".repeat(20_000)
    );
    assert_eq!(message(&deep), "this nests too deeply to parse");
    assert_eq!(
        help(&deep).as_deref(),
        Some("brackets nest at most 32 deep; name a part of it")
    );
}

#[test]
fn an_error_points_at_the_token_that_failed() {
    assert_eq!(
        render_error("import 1"),
        "7..8 expected a name, found a number\n"
    );
}

#[test]
fn only_a_name_is_assigned_to() {
    for written in [
        "    user.name = \"Bob\"",
        "    first(users).id = 1",
        "    2 + 2 = 4",
    ] {
        let source = format!("fn f() {{\n{written}\n}}");
        assert_eq!(
            message(&source),
            "only a name is assigned to",
            "{written} is refused"
        );
    }
}

#[test]
fn a_refused_assignment_names_the_one_way_to_change_a_value() {
    let source = "fn f() {\n    user.name = \"Bob\"\n}";
    assert_eq!(
        help(source).as_deref(),
        Some("build the value it becomes: `user { name: \"Bob\" }`")
    );
}

#[test]
fn the_refused_assignment_points_at_what_was_written_on_the_left() {
    let source = "fn f() {\n    user.name = 1\n}";
    let start = source.find("user").expect("the source writes the target");
    let expected = format!("{start}..{}", start + "user.name".len());
    let rendered = render_error(source);
    let span = rendered
        .lines()
        .next()
        .and_then(|line| line.split_once(' '))
        .expect("a rendered error opens with its span")
        .0;
    assert_eq!(
        span, expected,
        "the span covers the target, not the whole line"
    );
}

#[test]
fn an_underscore_is_written_on_the_left_of_a_single_equals_and_nowhere_else() {
    let refused = [
        ("fn f() {\n    _ += 1\n}\n", "expected `=`, found `+=`"),
        ("fn f() {\n    _ := 1\n}\n", "expected `=`, found `:=`"),
        ("fn f() {\n    _\n}\n", "expected `=`, found `}`"),
        ("fn f() {\n    var _ = 1\n}\n", "expected a name, found `_`"),
        (
            "fn f() {\n    go(_)\n}\n",
            "expected an expression, found `_`",
        ),
        (
            "fn f(users: List<Int>) {\n    for _ in users {\n        go()\n    }\n}\n",
            "expected an expression, found `_`",
        ),
    ];

    for (source, said) in refused {
        assert_eq!(message(source), said, "{source:?}");
    }
}

#[test]
fn a_trait_that_declares_no_method_is_a_parse_error() {
    assert_eq!(message("trait Eq<T> {\n}"), "expected `fn`, found `}`");
}

#[test]
fn an_instance_that_writes_no_method_is_a_parse_error() {
    assert_eq!(
        message("instance Eq<Point> {\n}"),
        "expected `fn`, found `}`"
    );
}

#[test]
fn an_instance_is_for_a_type_written_by_name_and_without_arguments() {
    assert_eq!(
        message(
            "instance Eq<List<Point>> {\n    fn is_equal(one, other) -> Bool {\n        true\n    }\n}"
        ),
        "expected `>`, found `<`"
    );
}

#[test]
fn a_trait_is_declared_over_one_type_parameter_and_no_more() {
    assert_eq!(
        message("trait Pair<T, U> {\n    fn first(both: T) -> U\n}"),
        "expected `>`, found `,`"
    );
}

#[test]
fn a_signature_of_a_trait_writes_no_body() {
    assert_eq!(
        message("trait Eq<T> {\n    fn is_equal(one: T, other: T) -> Bool {\n    }\n}"),
        "expected `fn`, found `{`"
    );
}

#[test]
fn a_field_written_with_a_parameter_is_a_parse_error_because_a_field_is_read() {
    assert_eq!(
        message("extern field out(count: Int) -> String = \"java.io.File.separator\"\n"),
        "expected `)`, found `count`"
    );
}

#[test]
fn a_method_written_with_no_parameter_at_all_has_no_receiver_to_be_called_on() {
    assert_eq!(
        message("extern method worded() -> String = \"toString\"\n"),
        "expected a name, found `)`"
    );
}
