//! `docs/specs/calls.md`: `maybe.or(fallback)` is the call `or(maybe, fallback)`.

use crate::common::{inferred, inferred_type, refusal};

/// A module declaring `held`, with `body` as the body of the function that calls it.
///
/// `held` takes an `Int` and a `String`, so nothing but the order holds its two apart and a call
/// of it is free to write either spelling.
fn calling_held(body: &str) -> String {
    format!(
        "fn go() -> String {{\n{body}}}\n\n\
         fn held(count: Int, said: String) -> String {{\n    said\n}}\n"
    )
}

/// A module declaring `rename`, with `call` as the call of it that `go` writes.
///
/// `rename` gives two of its parameters one type, so a call of it names its arguments and the
/// two ways of writing one are held to that.
fn calling_rename(call: &str) -> String {
    format!(
        "fn go() -> String {{\n    old := \"a\"\n    {call}\n}}\n\n\
         fn rename(from: String, to: String) -> String {{\n    from + to\n}}\n"
    )
}

#[test]
fn a_call_written_with_its_first_argument_in_front_is_the_call_it_is_written_as() {
    let source = calling_held("    count := 1\n    count.held(\"hi\")\n");

    assert_eq!(inferred_type(&source, "count.held(\"hi\")", 1), "String");
}

#[test]
fn the_name_after_the_dot_is_the_function_it_names_and_not_a_field() {
    let source = calling_held("    count := 1\n    count.held(\"hi\")\n");

    assert_eq!(
        inferred_type(&source, "count.held", 1),
        "(Int, String) -> String"
    );
}

#[test]
fn the_receiver_is_held_to_the_first_parameter_it_is_passed_for() {
    let source = calling_held("    said := \"hi\"\n    said.held(\"hi\")\n");

    assert_eq!(refusal(&source).message(), "expected `Int`, found `String`");
}

#[test]
fn the_receiver_counts_among_the_arguments_the_call_passes() {
    let source = calling_held("    count := 1\n    count.held()\n");

    assert_eq!(
        refusal(&source).message(),
        "`held` takes 2 arguments but 1 was given"
    );
}

#[test]
fn a_value_of_any_shape_at_all_stands_before_the_dot() {
    let source = calling_held("    held(1, \"hi\").held(2, \"there\")\n");

    assert_eq!(
        refusal(&source).message(),
        "`held` takes 2 arguments but 3 were given"
    );
}

#[test]
fn a_generic_function_written_in_front_form_is_instantiated_at_what_it_is_given() {
    let source = concat!(
        "fn go() -> Int {\n    count := 1\n    count.held(\"hi\")\n}\n\n",
        "fn held<T>(value: T, said: String) -> T {\n    value\n}\n"
    );

    assert_eq!(inferred_type(source, "count.held(\"hi\")", 1), "Int");
}

#[test]
fn a_trait_method_is_written_in_front_form_as_any_other_function_is() {
    let source = concat!(
        "derive Show for Payment\n\n",
        "fn said(payment: Payment) -> String {\n    payment.shown()\n}\n\n",
        "type Payment = Payment(Int)\n"
    );

    inferred(source);
}

#[test]
fn a_field_read_is_still_a_field_read_where_no_brackets_follow_it() {
    let source = "fn named(user: User) -> String {\n    user.name\n}\n\ntype User = {\n    name: String\n}\n";

    assert_eq!(inferred_type(source, "user.name", 1), "String");
}

#[test]
fn a_call_written_in_front_form_names_none_of_its_arguments() {
    let source = calling_held("    count := 1\n    count.held(said: \"hi\")\n");
    let error = refusal(&source);

    assert_eq!(
        error.message(),
        "`held` is written with its first argument in front, so this call names none of them"
    );
    assert_eq!(
        error.help(),
        "a call that names its arguments is written plainly, with every argument inside the brackets"
    );
}

#[test]
fn a_call_in_front_form_that_names_every_argument_is_still_refused_for_naming_them() {
    let source = calling_rename("old.rename(from: old, to: \"b\")");

    assert_eq!(
        refusal(&source).message(),
        "`rename` is written with its first argument in front, so this call names none of them"
    );
}

#[test]
fn a_call_that_has_to_name_its_arguments_is_refused_in_front_form_like_any_other() {
    let source = calling_rename("old.rename(\"b\")");

    assert_eq!(
        refusal(&source).message(),
        "`rename` gives two parameters the type `String`, so this call names its arguments"
    );
}
