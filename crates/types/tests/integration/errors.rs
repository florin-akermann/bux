//! The refusals of `docs/specs/types.md`, each in the words the reader is shown.

use crate::common::refusal;

#[test]
fn a_value_of_one_type_where_another_is_needed_is_refused() {
    let error = refusal("fn count() -> Int {\n    \"seven\"\n}\n");

    assert_eq!(error.message(), "expected `Int`, found `String`");
}

#[test]
fn a_newtype_is_not_the_type_it_wraps() {
    let source = "fn wrap(raw: Int) -> UserId {\n    raw\n}\n\ntype UserId = UserId(Int)\n";

    assert_eq!(refusal(source).message(), "expected `UserId`, found `Int`");
}

#[test]
fn a_generic_function_whose_body_fixes_its_parameter_is_refused() {
    let source = "fn identity<T>(value: T) -> T {\n    1\n}\n";

    assert_eq!(refusal(source).message(), "expected `T`, found `Int`");
}

#[test]
fn a_call_that_passes_too_many_arguments_is_refused() {
    let source = concat!(
        "fn go() -> Int {\n    twice(1, 2)\n}\n\n",
        "fn twice(value: Int) -> Int {\n    value\n}\n"
    );

    assert_eq!(
        refusal(source).message(),
        "`twice` takes 1 argument but 2 were given"
    );
}

#[test]
fn a_type_written_with_the_wrong_number_of_arguments_is_refused() {
    let source = "fn go(items: List<Int, Int>) -> Int {\n    1\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`List` takes 1 type argument but 2 were given"
    );
}

#[test]
fn a_field_a_record_does_not_declare_is_refused() {
    let source =
        "fn go(user: User) -> Bool {\n    user.busy\n}\n\ntype User = {\n    active: Bool\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`User` has no field named `busy`"
    );
}

#[test]
fn a_field_reached_through_a_type_nothing_settles_is_refused() {
    let source = "fn go(thing) -> Bool {\n    thing.active\n}\n";

    assert_eq!(
        refusal(source).message(),
        "the type here is not known, so `active` cannot be found"
    );
}

#[test]
fn a_record_built_without_one_of_its_fields_is_refused() {
    let source = concat!(
        "fn make() -> User {\n    User { id: 1 }\n}\n\n",
        "type User = {\n    id: Int\n    active: Bool\n}\n"
    );

    assert_eq!(
        refusal(source).message(),
        "`User` needs a field named `active`"
    );
}

#[test]
fn a_value_that_is_neither_an_int_nor_a_string_cannot_be_added() {
    let source = "fn go(flag: Bool) -> Bool {\n    flag + flag\n}\n";

    assert_eq!(refusal(source).message(), "`Bool` cannot be added");
}

#[test]
fn a_condition_that_is_not_a_bool_is_refused() {
    let source = "fn go(value: Int) -> Int {\n    if value {\n        1\n    } else {\n        2\n    }\n}\n";

    assert_eq!(refusal(source).message(), "expected `Bool`, found `Int`");
}

#[test]
fn a_for_in_over_something_that_is_not_a_list_is_refused() {
    let source =
        "fn go(value: Int) -> Int {\n    for item in value {\n        return 1\n    }\n    0\n}\n";

    assert_eq!(refusal(source).message(), "expected `List<_>`, found `Int`");
}

#[test]
fn a_question_mark_in_a_function_that_returns_no_result_is_refused() {
    let source = concat!(
        "fn go() -> Int {\n    name := find()?\n    1\n}\n\n",
        "fn find() -> Result<String, Int> {\n    Ok(\"found\")\n}\n"
    );

    assert_eq!(
        refusal(source).message(),
        "expected `Int`, found `Result<_, Int>`"
    );
}

#[test]
fn every_refusal_carries_the_help_line_its_code_has() {
    let error = refusal("fn count() -> Int {\n    \"seven\"\n}\n");

    assert_eq!(
        error.help(),
        "one type is not another, however alike they are held"
    );
}

#[test]
fn a_field_assigned_a_value_of_the_wrong_type_is_refused() {
    let source = concat!(
        "fn rename(user: User) -> Int {\n    user.id = \"one\"\n    user.id\n}\n\n",
        "type User = {\n    id: Int\n}\n"
    );

    assert_eq!(refusal(source).message(), "expected `Int`, found `String`");
}

#[test]
fn a_record_built_with_one_field_written_twice_is_refused() {
    let source = concat!(
        "fn make() -> User {\n    User { id: 1, id: 2 }\n}\n\n",
        "type User = {\n    id: Int\n}\n"
    );

    assert_eq!(refusal(source).message(), "`User` is given `id` twice");
}

#[test]
fn a_record_updated_with_one_field_written_twice_is_refused() {
    let source = concat!(
        "fn go(user: User) -> User {\n    user { active: true, active: false }\n}\n\n",
        "type User = {\n    active: Bool\n}\n"
    );

    assert_eq!(refusal(source).message(), "`user` is given `active` twice");
}

#[test]
fn a_variant_that_carries_its_value_in_order_has_no_field_to_write_against() {
    let source = concat!(
        "fn go() -> Payment {\n    Failed { reason: \"no\" }\n}\n\n",
        "type Payment =\n    | Pending\n    | Failed(String)\n"
    );

    assert_eq!(
        refusal(source).message(),
        "`Payment` has no field named `reason`"
    );
}

#[test]
fn an_unknown_field_of_a_generic_record_names_the_type_it_was_reached_through() {
    let source = concat!(
        "fn go(pair: Pair<Int>) -> Int {\n    pair.missing\n}\n\n",
        "type Pair<T> = {\n    one: T\n}\n"
    );

    assert_eq!(
        refusal(source).message(),
        "`Pair<Int>` has no field named `missing`"
    );
}

#[test]
fn a_field_used_where_another_type_is_needed_is_refused_where_the_other_is_written() {
    let source = concat!(
        "fn go(user: User) -> String {\n    user.name + 1\n}\n\n",
        "type User = {\n    name: String\n}\n"
    );

    assert_eq!(refusal(source).message(), "expected `String`, found `Int`");
}

#[test]
fn a_record_has_no_eq_so_two_of_them_are_not_compared() {
    let source = concat!(
        "fn same(left: User, right: User) -> Bool {\n    left == right\n}\n\n",
        "type User = {\n    id: Int\n}\n"
    );

    assert_eq!(
        refusal(source).message(),
        "`User` has no `Eq`, so two of them cannot be compared"
    );
}

#[test]
fn a_refused_comparison_points_at_the_whole_comparison_rather_than_at_one_side() {
    let source = concat!(
        "fn same(left: User, right: User) -> Bool {\n    left == right\n}\n\n",
        "type User = {\n    id: Int\n}\n"
    );

    assert_eq!(refusal(source).span().text(source), "left == right");
}

#[test]
fn a_variant_has_no_eq_either() {
    let source = concat!(
        "fn same(left: Payment, right: Payment) -> Bool {\n    left != right\n}\n\n",
        "type Payment =\n    | Pending\n    | Failed(String)\n"
    );

    assert_eq!(
        refusal(source).message(),
        "`Payment` has no `Eq`, so two of them cannot be compared"
    );
}

#[test]
fn a_unit_has_no_eq_because_no_instance_ships_for_it() {
    let source = "fn same(left: (), right: ()) -> Bool {\n    left == right\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`()` has no `Eq`, so two of them cannot be compared"
    );
}
