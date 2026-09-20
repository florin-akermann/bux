//! `docs/specs/arguments.md`: a parameter is never a bare `Bool`.

use crate::common::{inferred, refusal};

/// A module whose one function takes `takes` and gives back `gives`.
///
/// The body is `todo`, which takes whatever type is expected of it, so every module here differs
/// from the next in its signature alone, which is what the rule is about.
fn declaring(takes: &str, gives: &str) -> String {
    format!("fn open({takes}) -> {gives} {{\n    todo(\"the body is not the point\")\n}}\n")
}

#[test]
fn a_bool_parameter_says_what_a_call_of_it_would_read_like() {
    let error = refusal(&declaring("path: String, read_only: Bool", "String"));

    assert_eq!(
        error.message(),
        "this parameter is a `Bool`, so a call of `open` passes `true` and says no more"
    );
}

#[test]
fn the_help_says_what_to_declare_without_naming_a_function_the_program_does_not_have() {
    let error = refusal(&declaring("read_only: Bool", "String"));

    assert_eq!(
        error.help(),
        "declare a two-variant type and take that instead, so the call says which of the two"
    );
}

#[test]
fn a_two_variant_type_is_what_a_parameter_takes_instead() {
    let taking = "fn open(mode: Mode) -> String {\n    match mode {\n        ReadOnly => \"r\"\n        ReadWrite => \"rw\"\n    }\n}\n\n\
                  type Mode =\n    | ReadOnly\n    | ReadWrite\n";

    inferred(taking);
}

#[test]
fn a_boolean_operation_over_bool_keeps_its_parameters() {
    inferred("fn is_implied(first: Bool, second: Bool) -> Bool {\n    !first || second\n}\n");
    inferred("fn is_set(only: Bool) -> Bool {\n    only\n}\n");
}

#[test]
fn a_bool_among_other_types_is_a_flag_however_the_result_reads() {
    let error = refusal(&declaring("count: Int, loudly: Bool", "Bool"));

    assert_eq!(
        error.message(),
        "this parameter is a `Bool`, so a call of `open` passes `true` and says no more"
    );
}

#[test]
fn a_parameter_the_author_left_untyped_is_held_to_the_type_it_turned_out_to_have() {
    let inferred_bool =
        "fn open(flag) -> Int {\n    if flag {\n        1\n    } else {\n        2\n    }\n}\n";

    let error = refusal(inferred_bool);

    assert_eq!(
        error.message(),
        "this parameter is a `Bool`, so a call of `open` passes `true` and says no more"
    );
}

#[test]
fn a_type_parameter_is_not_a_bool_however_a_call_instantiates_it() {
    let generic = "fn is_passed() -> Bool {\n    open(true)\n}\n\n\
                   fn open<T>(value: T) -> T {\n    value\n}\n";

    inferred(generic);
}

#[test]
fn a_result_a_field_and_a_binding_may_each_be_a_bool() {
    let elsewhere = "fn is_open(user: User) -> Bool {\n    ready := user.active\n    ready\n}\n\n\
                     type User = {\n    active: Bool\n}\n";

    inferred(elsewhere);
}

#[test]
fn a_variant_that_carries_a_bool_is_not_a_parameter() {
    let carried = "fn open(answer: Answer) -> Answer {\n    answer\n}\n\n\
                   type Answer =\n    | Given(Bool)\n    | Missing\n";

    inferred(carried);
}

#[test]
fn the_refusal_points_at_the_parameter_because_the_declaration_is_what_changes() {
    let source = declaring("path: String, read_only: Bool", "String");

    let error = refusal(&source);

    assert_eq!(error.span().text(&source), "read_only: Bool");
}

#[test]
fn a_declaration_with_two_flags_is_refused_at_the_first_of_them() {
    let source = declaring("read_only: Bool, append: Bool", "String");

    let error = refusal(&source);

    assert_eq!(error.span().text(&source), "read_only: Bool");
}

#[test]
fn a_body_that_does_not_typecheck_is_refused_before_the_parameter_is_read() {
    let error = refusal("fn open(read_only: Bool) -> Int {\n    \"not an int\"\n}\n");

    assert_eq!(error.message(), "expected `Int`, found `String`");
}
