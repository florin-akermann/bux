//! The types `docs/specs/types.md` gives the version 0.1 expression forms.

use crate::common::inferred_type;

#[test]
fn a_whole_number_is_an_int() {
    let source = "fn count() -> Int {\n    7\n}\n";

    assert_eq!(inferred_type(source, "7", 1), "Int");
}

#[test]
fn a_quoted_string_is_a_string() {
    let source = "fn greeting() -> String {\n    \"hello\"\n}\n";

    assert_eq!(inferred_type(source, "\"hello\"", 1), "String");
}

#[test]
fn a_comparison_is_a_bool_however_it_is_written() {
    let source = "fn is_small(value: Int) -> Bool {\n    value < 10\n}\n";

    assert_eq!(inferred_type(source, "value < 10", 1), "Bool");
}

#[test]
fn a_binding_takes_the_type_of_what_it_is_given() {
    let source = "fn total() -> Int {\n    seen := 3\n    seen\n}\n";

    assert_eq!(inferred_type(source, "seen", 2), "Int");
}

#[test]
fn an_addition_of_strings_joins_them() {
    let source = "fn shout(word: String) -> String {\n    word + \"!\"\n}\n";

    assert_eq!(inferred_type(source, "word + \"!\"", 1), "String");
}

#[test]
fn an_addition_nothing_else_constrains_is_an_addition_of_ints() {
    let source = "fn twice(value: Int) -> Int {\n    value + value\n}\n";

    assert_eq!(inferred_type(source, "value", 2), "Int");
}

#[test]
fn a_for_loop_binds_the_item_of_the_list_it_walks() {
    let source = "fn first(words: List<String>) -> Int {\n    for word in words {\n        return 1\n    }\n    0\n}\n";

    assert_eq!(inferred_type(source, "words", 2), "List<String>");
}

#[test]
fn a_field_is_the_type_the_record_declares_it() {
    let source = "fn is_open(user: User) -> Bool {\n    user.active\n}\n\ntype User = {\n    active: Bool\n}\n";

    assert_eq!(inferred_type(source, "user.active", 1), "Bool");
}

#[test]
fn a_record_is_built_at_the_type_it_declares() {
    let source =
        "fn make() -> User {\n    User { active: true }\n}\n\ntype User = {\n    active: Bool\n}\n";

    assert_eq!(inferred_type(source, "User { active: true }", 1), "User");
}

#[test]
fn a_record_update_has_the_type_it_already_had() {
    let source = "fn close(user: User) -> User {\n    user { active: false }\n}\n\ntype User = {\n    active: Bool\n}\n";

    assert_eq!(inferred_type(source, "user { active: false }", 1), "User");
}

#[test]
fn a_question_mark_gives_the_ok_and_leaves_with_the_error() {
    let source = concat!(
        "fn load(id: Int) -> Result<String, Int> {\n    name := find(id)?\n    Ok(name)\n}\n\n",
        "fn find(id: Int) -> Result<String, Int> {\n    Ok(\"found\")\n}\n"
    );

    assert_eq!(inferred_type(source, "find(id)?", 1), "String");
}

#[test]
fn a_match_arm_binds_what_its_variant_carries() {
    let source = concat!(
        "fn describe(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\n",
        "type Payment =\n    | Pending\n    | Failed(String)\n"
    );

    assert_eq!(inferred_type(source, "reason", 2), "String");
}

#[test]
fn a_newtype_is_the_type_it_declares_and_not_the_one_it_wraps() {
    let source = "fn wrap(raw: Int) -> UserId {\n    UserId(raw)\n}\n\ntype UserId = UserId(Int)\n";

    assert_eq!(inferred_type(source, "UserId(raw)", 1), "UserId");
}

#[test]
fn an_if_with_an_else_is_the_type_both_of_its_branches_have() {
    let source = "fn pick(count: Int) -> Int {\n    if count > 1 {\n        1\n    } else {\n        2\n    }\n}\n";

    assert_eq!(
        inferred_type(
            source,
            "if count > 1 {\n        1\n    } else {\n        2\n    }",
            1
        ),
        "Int"
    );
}

#[test]
fn a_name_bound_to_a_field_keeps_the_type_the_record_declares() {
    let source = concat!(
        "fn go(user: User) -> String {\n    found := user.name\n    found\n}\n\n",
        "type User = {\n    name: String\n}\n"
    );

    assert_eq!(inferred_type(source, "found", 2), "String");
}

#[test]
fn a_field_reached_through_a_name_bound_to_a_field_is_found() {
    let source = concat!(
        "fn go(outer: Outer) -> Int {\n    held := outer.inner\n    held.depth\n}\n\n",
        "type Outer = {\n    inner: Inner\n}\n\n",
        "type Inner = {\n    depth: Int\n}\n"
    );

    assert_eq!(inferred_type(source, "held.depth", 1), "Int");
}

#[test]
fn the_three_types_the_library_ships_an_eq_for_are_compared() {
    let source = concat!(
        "fn is_same(left: String, right: String) -> Bool {\n",
        "    1 == 2 && true == false && left == right\n",
        "}\n"
    );

    assert_eq!(inferred_type(source, "left == right", 1), "Bool");
}

#[test]
fn a_comparison_of_two_numbers_nothing_else_pins_down_is_a_comparison_of_ints() {
    let source = "fn is_same(left: Int) -> Bool {\n    left == 1\n}\n";

    assert_eq!(inferred_type(source, "1", 1), "Int");
}

#[test]
fn a_division_is_an_option_because_a_zero_divisor_has_no_answer() {
    let source = "fn share(total: Int, count: Int) -> Option<Int> {\n    total / count\n}\n";

    assert_eq!(inferred_type(source, "total / count", 1), "Option<Int>");
}

#[test]
fn a_remainder_is_an_option_for_the_same_reason() {
    let source = "fn left(total: Int, count: Int) -> Option<Int> {\n    total % count\n}\n";

    assert_eq!(inferred_type(source, "total % count", 1), "Option<Int>");
}

#[test]
fn a_call_of_or_gives_back_what_the_option_it_is_handed_holds() {
    let source = "fn share(total: Int, count: Int) -> Int {\n    or(total / count, 0)\n}\n";

    assert_eq!(inferred_type(source, "or(total / count, 0)", 1), "Int");
}
