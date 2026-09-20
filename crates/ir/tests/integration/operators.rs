//! What an operator becomes, which `docs/specs/operators.md` states.
//!
//! An operator over a type that has an instance is a call of that instance's method; an operator
//! over `Int` is the instruction it has always been, because the prelude's instance is that
//! instruction rather than a method written in Lumen.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{Arithmetic, ClassName, Comparison, Instruction};

use crate::common;

/// A declared type owning every operator a type can own, and a use of each.
const MONEY: &str = concat!(
    "fn worked_out(one: Money, other: Money) -> Money {\n    one + other\n}\n\n",
    "fn is_cheaper(one: Money, other: Money) -> Bool {\n    one < other\n}\n\n",
    "fn is_dearer(one: Money, other: Money) -> Bool {\n    one > other\n}\n\n",
    "fn owed(money: Money) -> Money {\n    -money\n}\n\n",
    "instance Neg<Money> {\n    fn negate(value: Money) -> Money {\n        value\n    }\n}\n\n",
    "instance Ord<Money> {\n    fn is_less(one: Money, other: Money) -> Bool {\n",
    "        one.cents < other.cents\n    }\n}\n\n",
    "instance Add<Money> {\n    fn add(one: Money, other: Money) -> Money {\n",
    "        Money { cents: one.cents + other.cents }\n    }\n}\n\n",
    "type Money = {\n    cents: Int\n}\n"
);

/// `Money` with an instance of every trait an operator is, so any operator may be written over it.
const MONEY_OWNS: &str = concat!(
    "\ninstance Add<Money> {\n    fn add(one: Money, other: Money) -> Money {\n        one\n    }\n}\n\n",
    "instance Sub<Money> {\n    fn subtract(one: Money, other: Money) -> Money {\n        one\n    }\n}\n\n",
    "instance Mul<Money> {\n    fn multiply(one: Money, other: Money) -> Money {\n        one\n    }\n}\n\n",
    "instance Div<Money> {\n    fn divide(one: Money, other: Money) -> Option<Money> {\n",
    "        Some(one)\n    }\n}\n\n",
    "instance Rem<Money> {\n    fn remainder(one: Money, other: Money) -> Option<Money> {\n",
    "        Some(one)\n    }\n}\n\n",
    "instance Neg<Money> {\n    fn negate(value: Money) -> Money {\n        value\n    }\n}\n\n",
    "instance Ord<Money> {\n    fn is_less(one: Money, other: Money) -> Bool {\n",
    "        one.cents < other.cents\n    }\n}\n\n",
    "type Money = {\n    cents: Int\n}\n"
);

/// The module every example here is lowered as.
fn demo() -> ClassName {
    ClassName::new("demo")
}

#[test]
fn a_plus_over_a_type_with_an_instance_of_add_calls_that_instance() {
    let lowered = common::lowered(MONEY);

    assert!(common::calls(
        common::body_of(&lowered, "worked_out"),
        &demo(),
        "Add$Money$add"
    ));
}

#[test]
fn a_prefix_minus_over_a_type_with_an_instance_of_neg_calls_that_instance() {
    let lowered = common::lowered(MONEY);

    assert!(common::calls(
        common::body_of(&lowered, "owed"),
        &demo(),
        "Neg$Money$negate"
    ));
}

#[test]
fn a_less_than_over_a_type_with_an_instance_of_ord_calls_that_instance() {
    let lowered = common::lowered(MONEY);

    assert!(common::calls(
        common::body_of(&lowered, "is_cheaper"),
        &demo(),
        "Ord$Money$is_less"
    ));
}

#[test]
fn a_greater_than_is_the_same_is_less_called_with_its_operands_the_other_way_round() {
    let lowered = common::lowered(MONEY);

    let dearer = common::body_of(&lowered, "is_dearer");

    assert!(common::calls(dearer, &demo(), "Ord$Money$is_less"));
    assert!(
        !dearer.instructions.contains(&Instruction::Not),
        "`>` is `is_less` the other way round rather than the negation of one"
    );
}

#[test]
fn a_greater_or_equal_is_a_less_than_negated() {
    let source = concat!(
        "fn is_enough(one: Money, other: Money) -> Bool {\n    one >= other\n}\n\n",
        "instance Ord<Money> {\n    fn is_less(one: Money, other: Money) -> Bool {\n",
        "        one.cents < other.cents\n    }\n}\n\n",
        "type Money = {\n    cents: Int\n}\n"
    );

    let lowered = common::lowered(source);
    let enough = common::body_of(&lowered, "is_enough");

    assert!(common::calls(enough, &demo(), "Ord$Money$is_less"));
    assert!(enough.instructions.contains(&Instruction::Not));
}

#[test]
fn a_plus_equals_over_a_type_with_an_instance_of_add_calls_that_instance() {
    let source = concat!(
        "fn worked_out(one: Money, other: Money) -> Money {\n",
        "    var total = one\n    total += other\n    total\n}\n\n",
        "instance Add<Money> {\n    fn add(one: Money, other: Money) -> Money {\n",
        "        Money { cents: one.cents + other.cents }\n    }\n}\n\n",
        "type Money = {\n    cents: Int\n}\n"
    );

    let lowered = common::lowered(source);
    let worked_out = common::body_of(&lowered, "worked_out");

    assert!(common::calls(worked_out, &demo(), "Add$Money$add"));
    assert!(
        !worked_out.instructions.contains(&Instruction::Concat),
        "`+=` is the instance's `add`, not the join the prelude's `Add` for `String` writes"
    );
}

#[test]
fn a_plus_equals_over_ints_stays_the_addition_it_has_always_been() {
    let source = concat!(
        "fn worked_out(one: Int, other: Int) -> Int {\n",
        "    var total = one\n    total += other\n    total\n}\n"
    );

    let lowered = common::lowered(source);

    assert!(
        common::body_of(&lowered, "worked_out")
            .instructions
            .contains(&Instruction::Arithmetic(Arithmetic::Add))
    );
}

#[test]
fn an_operator_over_ints_stays_the_instruction_it_has_always_been() {
    let source = concat!(
        "fn worked_out(one: Int, other: Int) -> Int {\n    one * other - one\n}\n\n",
        "fn is_cheaper(one: Int, other: Int) -> Bool {\n    one < other\n}\n"
    );

    let lowered = common::lowered(source);

    let instructions = &common::body_of(&lowered, "worked_out").instructions;
    assert!(instructions.contains(&Instruction::Arithmetic(Arithmetic::Multiply)));
    assert!(instructions.contains(&Instruction::Arithmetic(Arithmetic::Subtract)));
    assert!(
        common::body_of(&lowered, "is_cheaper")
            .instructions
            .contains(&Instruction::CompareLongs(Comparison::Less))
    );
}

#[test]
fn a_generic_constrained_by_add_calls_the_instance_of_the_type_it_is_specialized_at() {
    let source = concat!(
        "fn worked_out(one: Money, other: Money) -> Money {\n    joined(one: one, other: other)\n}\n\n",
        "fn joined<T: Add<T>>(one: T, other: T) -> T {\n    one + other\n}\n\n",
        "instance Add<Money> {\n    fn add(one: Money, other: Money) -> Money {\n",
        "        Money { cents: one.cents + other.cents }\n    }\n}\n\n",
        "type Money = {\n    cents: Int\n}\n"
    );

    let lowered = common::lowered(source);

    assert!(common::calls(
        common::body_of(&lowered, "joined$Money"),
        &demo(),
        "Add$Money$add"
    ));
}

/// Each operator, the way it is written over two `Money`s, and the instance method it is.
const OVER_MONEY: [(&str, &str); 10] = [
    ("one + other", "Add$Money$add"),
    ("one - other", "Sub$Money$subtract"),
    ("one * other", "Mul$Money$multiply"),
    ("one / other", "Div$Money$divide"),
    ("one % other", "Rem$Money$remainder"),
    ("-one", "Neg$Money$negate"),
    ("one < other", "Ord$Money$is_less"),
    ("one <= other", "Ord$Money$is_less"),
    ("one > other", "Ord$Money$is_less"),
    ("one >= other", "Ord$Money$is_less"),
];

/// An operator over a type with an instance calls that instance and no other.
#[hegel::test]
fn an_operator_over_a_type_with_an_instance_calls_that_instance_and_no_other(tc: TestCase) {
    let (written, called) = tc.draw(gs::sampled_from(&OVER_MONEY));

    let lowered = common::lowered(&owning(written));
    let reached = instances_called(&lowered);

    assert_eq!(reached, vec![called]);
}

/// Each of the four comparisons is one call of the instance's `is_less`, however it is written.
#[hegel::test]
fn a_comparison_is_one_call_of_is_less_whichever_way_round_it_is_written(tc: TestCase) {
    let written = tc.draw(gs::sampled_from(&[
        "one < other",
        "one <= other",
        "one > other",
        "one >= other",
    ]));

    let lowered = common::lowered(&owning(written));

    assert_eq!(instances_called(&lowered), vec!["Ord$Money$is_less"]);
}

/// A module writing `written` over `Money`, which has an instance of every operator's trait.
fn owning(written: &str) -> String {
    format!(
        "fn worked_out(one: Money, other: Money) -> () {{\n    _ = {written}\n    ()\n}}\n{MONEY_OWNS}"
    )
}

/// The instance methods `worked_out` calls, in the order it calls them.
fn instances_called(lowered: &lumen_ir::Lowered) -> Vec<&str> {
    common::body_of(lowered, "worked_out")
        .instructions
        .iter()
        .filter_map(common::called)
        .map(|reference| reference.name.as_str())
        .filter(|name| name.contains('$'))
        .collect()
}
