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

/// A module declaring a trait whose one method takes a `T`, and an instance of it for `Bool`.
///
/// The instance writes `Bool` because the trait wrote `T` and the instance is for `Bool`; the
/// types are the trait's and not the author's, which is what the rule turns on.
fn answering(method: &str) -> String {
    format!(
        "instance Asked<Bool> {{\n    {method}\n}}\n\n\
         trait Asked<T> {{\n    fn answered(value: T) -> Int\n}}\n"
    )
}

#[test]
fn an_instance_method_keeps_a_bool_parameter_its_trait_gave_it() {
    inferred(&answering(
        "fn answered(value: Bool) -> Int {\n        1\n    }",
    ));
}

#[test]
fn a_bool_a_trait_wrote_outright_is_a_flag_where_the_trait_wrote_it() {
    let source = concat!(
        "instance Asked<Int> {\n    fn answered(value: Bool, at: Int) -> Int {\n        1\n    }\n}\n\n",
        "trait Asked<T> {\n    fn answered(value: Bool, at: T) -> Int\n}\n"
    );

    let error = refusal(source);

    assert_eq!(
        error.message(),
        "this parameter is a `Bool`, so a call of `answered` passes `true` and says no more"
    );
    assert_eq!(error.span().text(source), "value: Bool");
}

#[test]
fn a_trait_method_that_is_all_bool_keeps_its_parameters_as_a_function_would() {
    let source = concat!(
        "instance Asked<Bool> {\n",
        "    fn is_answered(value: Bool) -> Bool {\n        value\n    }\n}\n\n",
        "trait Asked<T> {\n    fn is_answered(value: Bool) -> Bool\n}\n"
    );

    inferred(source);
}

#[test]
fn the_same_signature_written_as_a_function_is_a_flag_all_the_same() {
    let error = refusal("fn answered(value: Bool) -> Int {\n    1\n}\n");

    assert_eq!(
        error.message(),
        "this parameter is a `Bool`, so a call of `answered` passes `true` and says no more"
    );
}

#[test]
fn an_instance_method_is_still_held_to_the_signature_its_trait_gave_it() {
    let error = refusal(&answering(
        "fn answered(value: Bool) -> String {\n        \"one\"\n    }",
    ));

    assert_eq!(
        error.message(),
        "expected `(Bool) -> Int`, found `(Bool) -> String`"
    );
}
