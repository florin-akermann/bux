//! Every operator is a trait method, which `docs/specs/operators.md` states.
//!
//! What each operator gives back over `Int` is what it gave back before it was a trait, and a
//! declared type gets an operator the one way any type gets one: by writing that trait's instance.

use hegel::TestCase;
use hegel::generators as gs;

use crate::common::{inferred_type, refusal};

/// A declared type with an instance of `Add`, of `Ord`, and of `Neg`, and a `+` over it.
const MONEY: &str = concat!(
    "fn worked_out(one: Money, other: Money) -> Money {\n    one + other\n}\n\n",
    "instance Neg<Money> {\n    fn negate(value: Money) -> Money {\n        value\n    }\n}\n\n",
    "instance Ord<Money> {\n    fn is_less(one: Money, other: Money) -> Bool {\n",
    "        one.cents < other.cents\n    }\n}\n\n",
    "instance Add<Money> {\n    fn add(one: Money, other: Money) -> Money {\n",
    "        Money { cents: one.cents + other.cents }\n    }\n}\n\n",
    "type Money = {\n    cents: Int\n}\n"
);

/// A declared type that has an instance of nothing, so every operator over it is refused.
const POINT: &str = "\ntype Point = {\n    across: Int\n}\n";

/// Each operator, the way it is written between two `Int`s, and the type it gives back.
const OVER_INTS: [(&str, &str); 9] = [
    ("one + other", "Int"),
    ("one - other", "Int"),
    ("one * other", "Int"),
    ("one / other", "Option<Int>"),
    ("one % other", "Option<Int>"),
    ("one < other", "Bool"),
    ("one <= other", "Bool"),
    ("one > other", "Bool"),
    ("one >= other", "Bool"),
];

#[test]
fn an_operator_over_a_declared_type_gives_back_what_its_instance_gives_back() {
    assert_eq!(inferred_type(MONEY, "one + other", 1), "Money");
}

#[test]
fn a_comparison_over_a_declared_type_with_an_instance_of_ord_gives_a_bool() {
    let source = concat!(
        "fn is_cheaper(one: Money, other: Money) -> Bool {\n    one < other\n}\n\n",
        "instance Ord<Money> {\n    fn is_less(one: Money, other: Money) -> Bool {\n",
        "        one.cents < other.cents\n    }\n}\n\n",
        "type Money = {\n    cents: Int\n}\n"
    );

    assert_eq!(inferred_type(source, "one < other", 1), "Bool");
}

#[test]
fn a_prefix_minus_over_a_declared_type_with_an_instance_of_neg_gives_that_type() {
    let source = concat!(
        "fn owed(money: Money) -> Money {\n    -money\n}\n\n",
        "instance Neg<Money> {\n    fn negate(value: Money) -> Money {\n        value\n    }\n}\n\n",
        "type Money = {\n    cents: Int\n}\n"
    );

    assert_eq!(inferred_type(source, "-money", 1), "Money");
}

#[test]
fn a_division_over_a_declared_type_gives_back_an_option_of_it() {
    let source = concat!(
        "fn split(one: Money, other: Money) -> Option<Money> {\n    one / other\n}\n\n",
        "instance Div<Money> {\n    fn divide(one: Money, other: Money) -> Option<Money> {\n",
        "        Some(one)\n    }\n}\n\n",
        "type Money = {\n    cents: Int\n}\n"
    );

    assert_eq!(inferred_type(source, "one / other", 1), "Option<Money>");
}

#[test]
fn a_generic_constrained_by_add_may_write_a_plus_over_its_type_parameter() {
    let source = "fn joined<T: Add<T>>(one: T, other: T) -> T {\n    one + other\n}\n";

    assert_eq!(inferred_type(source, "one + other", 1), "T");
}

#[test]
fn a_generic_that_promises_nothing_may_not_write_an_operator() {
    let source = "fn joined<T>(one: T, other: T) -> T {\n    one + other\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`T` has no `Add`, so `+` is not written over it"
    );
}

#[test]
fn a_string_has_add_so_two_of_them_are_joined_with_a_plus() {
    let source = "fn joined(one: String, other: String) -> String {\n    one + other\n}\n";

    assert_eq!(inferred_type(source, "one + other", 1), "String");
}

#[test]
fn the_prelude_orders_a_string_and_a_bool_as_it_orders_an_int() {
    for held in ["String", "Bool"] {
        let source =
            format!("fn is_first(one: {held}, other: {held}) -> Bool {{\n    one < other\n}}\n");

        assert_eq!(inferred_type(&source, "one < other", 1), "Bool", "{held}");
    }
}

#[test]
fn a_type_with_no_ord_is_not_compared_and_is_told_to_derive_one() {
    let source = concat!(
        "fn is_first(one: Money, other: Money) -> Bool {\n    one < other\n}\n\n",
        "type Money = {\n    cents: Int\n}\n"
    );

    assert_eq!(
        refusal(source).message(),
        "`Money` has no `Ord`, so `<` is not written over it"
    );
    assert_eq!(
        refusal(source).help(),
        "`Money` gets one by deriving it: write `derive Ord for Money`"
    );
}

#[test]
fn a_plus_equals_over_a_declared_type_with_an_add_instance_is_accepted() {
    let source = concat!(
        "fn worked_out(one: Money, other: Money) -> Money {\n",
        "    var total = one\n    total += other\n    total\n}\n\n",
        "instance Add<Money> {\n    fn add(one: Money, other: Money) -> Money {\n",
        "        Money { cents: one.cents + other.cents }\n    }\n}\n\n",
        "type Money = {\n    cents: Int\n}\n"
    );

    assert_eq!(inferred_type(source, "total", 3), "Money");
}

#[test]
fn a_plus_equals_asks_add_of_what_it_is_assigned_to() {
    let source =
        "fn counted(flag: Bool) -> Bool {\n    var seen = flag\n    seen += flag\n    seen\n}\n";

    assert_eq!(
        refusal(source).message(),
        "`Bool` has no `Add`, so `+=` is not written over it"
    );
}

/// Every operator over two `Int`s gives back the type it gave back before it was a trait.
#[hegel::test]
fn an_operator_over_ints_gives_back_what_it_always_gave(tc: TestCase) {
    let (written, gives) = tc.draw(gs::sampled_from(&OVER_INTS));
    let source = written_over(written, "Int", "");

    assert_eq!(inferred_type(&source, written, 1), *gives);
}

/// An operator over a type that has no instance of its trait is refused, whichever it is.
#[hegel::test]
fn an_operator_over_a_type_with_no_instance_is_refused(tc: TestCase) {
    let (written, _) = tc.draw(gs::sampled_from(&OVER_INTS));
    let source = written_over(written, "Point", POINT);

    assert!(refusal(&source).message().starts_with("`Point` has no `"));
}

/// A module writing `written` over two values of `over`, beside whatever `declares` declares.
///
/// The operator is thrown away rather than given back, so the module says nothing about the type
/// it settles on beyond the type of what it is written over.
fn written_over(written: &str, over: &str, declares: &str) -> String {
    format!(
        "fn worked_out(one: {over}, other: {over}) -> () {{\n    _ = {written}\n    ()\n}}\n{declares}"
    )
}
