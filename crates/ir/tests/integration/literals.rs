//! What a whole-number literal becomes, which `docs/specs/literals.md` states.
//!
//! `Int`, whose instance the prelude writes, takes the number itself, which is what a literal
//! has always been. A type whose instance a module wrote takes it through that instance's
//! `from_literal`, so the number is pushed and one call takes it.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{Instruction, Lowered};

use crate::common;

/// A declared type that takes a whole number, and the instance saying how it takes one.
const INT32: &str = concat!(
    "\ninstance IntegerLiteral<Int32> {\n",
    "    fn lowest() -> Int {\n        -2147483648\n    }\n\n",
    "    fn highest() -> Int {\n        2147483647\n    }\n\n",
    "    fn from_literal(literal: Int) -> Int32 {\n        Int32(literal)\n    }\n}\n\n",
    "type Int32 = Int32(Int)\n"
);

/// The one call a literal at `Int32` costs.
const FROM_LITERAL: &str = "IntegerLiteral$Int32$from_literal";

/// Places a whole number is written, each of which settles it on the declared `Int32`.
const AT_INT32: [&str; 3] = ["42", "-1", "2147483647"];

#[test]
fn a_literal_at_a_type_whose_instance_a_module_wrote_calls_that_instances_from_literal() {
    let lowered = common::lowered(&counted("42"));

    assert_eq!(calls_of(&lowered), vec![FROM_LITERAL]);
}

#[test]
fn the_number_is_pushed_before_the_call_that_takes_it() {
    let lowered = common::lowered(&counted("42"));
    let instructions = &common::body_of(&lowered, "counted").instructions;

    assert_eq!(instructions.first(), Some(&Instruction::Long(42)));
}

#[test]
fn a_literal_at_int_is_the_number_itself_and_costs_no_call() {
    let lowered = common::lowered("fn counted() -> Int {\n    42\n}\n");
    let instructions = &common::body_of(&lowered, "counted").instructions;

    assert!(instructions.contains(&Instruction::Long(42)));
    assert!(calls_of(&lowered).is_empty());
}

#[test]
fn a_literal_in_an_argument_is_taken_by_the_instance_of_the_type_the_parameter_declares() {
    let source = format!(
        "fn counted() -> Int32 {{\n    passed(count: 42)\n}}\n\nfn passed(count: Int32) -> Int32 {{\n    count\n}}\n{INT32}"
    );
    let lowered = common::lowered(&source);

    assert_eq!(calls_of(&lowered), vec![FROM_LITERAL, "passed"]);
}

#[test]
fn from_literal_at_int_is_the_whole_number_it_is_given_and_costs_no_call() {
    let lowered = common::lowered("fn counted() -> Int {\n    from_literal(7)\n}\n");
    let instructions = &common::body_of(&lowered, "counted").instructions;

    assert_eq!(instructions.first(), Some(&Instruction::Long(7)));
    assert!(calls_of(&lowered).is_empty());
}

#[test]
fn a_name_bound_to_a_literal_takes_it_through_the_instance_of_the_type_its_use_expects() {
    let source = format!("fn counted() -> Int32 {{\n    count := 42\n    count\n}}\n{INT32}");
    let lowered = common::lowered(&source);

    assert_eq!(calls_of(&lowered), vec![FROM_LITERAL]);
}

#[test]
fn the_bounds_are_written_out_like_any_other_instance_method() {
    let lowered = common::lowered(&counted("42"));

    assert!(common::has_method(&lowered, "IntegerLiteral$Int32$lowest"));
    assert!(common::has_method(&lowered, "IntegerLiteral$Int32$highest"));
}

/// A literal at a type whose instance a module wrote is one call of that instance, wherever it is.
#[hegel::test]
fn a_literal_at_a_declared_type_is_one_call_of_its_from_literal(tc: TestCase) {
    let written = tc.draw(gs::sampled_from(&AT_INT32));
    let lowered = common::lowered(&counted(written));

    assert_eq!(calls_of(&lowered), vec![FROM_LITERAL]);
}

/// A whole number written where an `Int` is expected lowers exactly as it did before.
#[hegel::test]
fn a_whole_number_where_an_int_is_expected_is_that_number_and_nothing_more(tc: TestCase) {
    let value: i64 = tc.draw(gs::integers().min_value(0).max_value(i64::MAX));
    let lowered = common::lowered(&format!("fn counted() -> Int {{\n    {value}\n}}\n"));
    let instructions = &common::body_of(&lowered, "counted").instructions;

    assert!(instructions.contains(&Instruction::Long(value)));
    assert!(calls_of(&lowered).is_empty());
}

/// A module whose `counted` gives back the `Int32` that `written` reads as.
fn counted(written: &str) -> String {
    format!("fn counted() -> Int32 {{\n    {written}\n}}\n{INT32}")
}

/// The methods `counted` calls, in the order it calls them.
fn calls_of(lowered: &Lowered) -> Vec<&str> {
    common::body_of(lowered, "counted")
        .instructions
        .iter()
        .filter_map(common::called)
        .map(|reference| reference.name.as_str())
        .collect()
}
