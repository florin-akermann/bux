//! Parametric polymorphism: a generic name is fresh at every place it is used.

use crate::common::{inferred_type, refusal};

/// The source both of the generic tests work from: one generic function used at two types.
const AT_TWO_TYPES: &str = concat!(
    "fn go() -> String {\n    number := identity(1)\n    word := identity(\"two\")\n    word\n}\n\n",
    "fn identity<T>(value: T) -> T {\n    value\n}\n"
);

#[test]
fn a_generic_function_is_used_at_a_type_of_its_own_each_time() {
    assert_eq!(inferred_type(AT_TWO_TYPES, "identity(1)", 1), "Int");
    assert_eq!(
        inferred_type(AT_TWO_TYPES, "identity(\"two\")", 1),
        "String"
    );
}

#[test]
fn a_generic_type_carries_whatever_it_was_built_with() {
    let source = "fn wrap(word: String) -> Option<String> {\n    Some(word)\n}\n";

    assert_eq!(inferred_type(source, "Some(word)", 1), "Option<String>");
}

#[test]
fn a_binding_is_as_polymorphic_as_the_value_it_was_given() {
    let source = concat!(
        "fn go() -> String {\n    nothing := none_of()\n    number := unwrap_or(nothing, 0)\n    word := unwrap_or(nothing, \"none\")\n    word\n}\n\n",
        "fn unwrap_or<T>(value: Option<T>, fallback: T) -> T {\n    match value {\n        Some(inner) => inner\n        None => fallback\n    }\n}\n\n",
        "fn none_of<T>() -> Option<T> {\n    None\n}\n"
    );

    assert_eq!(inferred_type(source, "unwrap_or(nothing, 0)", 1), "Int");
    assert_eq!(
        inferred_type(source, "unwrap_or(nothing, \"none\")", 1),
        "String"
    );
}

#[test]
fn a_generic_type_used_at_two_types_at_once_is_refused() {
    let source = concat!(
        "fn go(word: String) -> Int {\n    identity(word)\n}\n\n",
        "fn identity<T>(value: T) -> T {\n    value\n}\n"
    );

    assert_eq!(refusal(source).message(), "expected `Int`, found `String`");
}

#[test]
fn a_generic_record_carries_the_type_it_was_declared_with() {
    let source = concat!(
        "fn unwrap(boxed: Box<Int>) -> Int {\n    boxed.value\n}\n\n",
        "type Box<T> = {\n    value: T\n}\n"
    );

    assert_eq!(inferred_type(source, "boxed.value", 1), "Int");
}

#[test]
fn a_function_with_no_signature_is_generalised_over_what_its_body_left_free() {
    let source = concat!(
        "fn go() -> String {\n    number := same(1)\n    word := same(\"two\")\n    if number > 0 {\n        return word\n    }\n    word\n}\n\n",
        "fn same(value) {\n    value\n}\n"
    );

    assert_eq!(inferred_type(source, "same(1)", 1), "Int");
    assert_eq!(inferred_type(source, "same(\"two\")", 1), "String");
}
