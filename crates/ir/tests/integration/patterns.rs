//! `docs/specs/patterns.md`: what `_`, a literal, and an or-pattern each become.

use lumen_ir::{ClassName, Comparison, Instruction, Lowered};

use crate::common::{body_of, lowered};

#[test]
fn an_underscore_tests_nothing_and_stores_nothing() {
    let named = lowered(&matching("        Red => 1\n        rest => 2\n"));
    let ignored = lowered(&matching("        Red => 1\n        _ => 2\n"));

    assert_eq!(
        counted(&named, is_a_store),
        counted(&ignored, is_a_store) + 1
    );
}

#[test]
fn an_arm_of_alternatives_answers_for_what_either_of_them_answers_for() {
    let (together, apart) = together_and_apart();

    assert_eq!(counted(&together, is_a_test), counted(&apart, is_a_test));
}

#[test]
fn an_arm_of_alternatives_writes_its_body_once_where_two_arms_write_it_twice() {
    let (together, apart) = together_and_apart();

    assert_eq!(counted(&together, is_the_answer), 1);
    assert_eq!(counted(&apart, is_the_answer), 2);
}

#[test]
fn a_whole_number_pattern_over_an_int_compares_the_number_itself() {
    let source = "fn counted(count: Int) -> Int {\n    match count {\n        0 => 1\n        _ => 2\n    }\n}\n";
    let counting = lowered(source);

    let instructions = &body_of(&counting, "counted").instructions;
    assert!(
        instructions.contains(&Instruction::Long(0)),
        "{instructions:?}"
    );
}

#[test]
fn a_whole_number_pattern_over_a_declared_type_goes_through_its_two_instances() {
    let source = concat!(
        "derive Eq for Int32\n\n",
        "fn counted(count: Int32) -> Int {\n    match count {\n        5 => 1\n        _ => 2\n    }\n}\n\n",
        "instance IntegerLiteral<Int32> {\n",
        "    fn lowest() -> Int {\n        -2147483648\n    }\n\n",
        "    fn highest() -> Int {\n        2147483647\n    }\n\n",
        "    fn from_literal(literal: Int) -> Int32 {\n        Int32(literal)\n    }\n}\n\n",
        "type Int32 = Int32(Int)\n"
    );
    let counting = lowered(source);

    let called: Vec<&str> = body_of(&counting, "counted")
        .instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::InvokeStatic(method) => Some(method.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        called,
        ["IntegerLiteral$Int32$from_literal", "Eq$Int32$is_equal"]
    );
}

#[test]
fn a_whole_number_inside_what_a_variant_carries_is_compared_at_the_type_it_carries() {
    let source = concat!(
        "fn described(held: Box<Int>) -> Int {\n    match held {\n",
        "        Box(1) => 1\n        Box(_) => 2\n    }\n}\n\n",
        "type Box<T> = Box(T)\n"
    );
    let holding = lowered(source);

    let instructions = &body_of(&holding, "described").instructions;
    let compared = instructions
        .iter()
        .position(|instruction| instruction == &Instruction::CompareLongs(Comparison::Equal))
        .expect("the two numbers are compared as the numbers they are");
    assert!(
        instructions[..compared].contains(&Instruction::Cast(ClassName::new("java/lang/Long"))),
        "{instructions:?}"
    );
}

/// The one answer written as two alternatives, and the same answer written as one arm each.
fn together_and_apart() -> (Lowered, Lowered) {
    (
        lowered(&matching("        Red | Green => 1\n        Blue => 2\n")),
        lowered(&matching(
            "        Red => 1\n        Green => 1\n        Blue => 2\n",
        )),
    )
}

/// A module matching a colour, with `arms` as the arms of the one `match`.
fn matching(arms: &str) -> String {
    format!(
        "fn described(colour: Colour) -> Int {{\n    match colour {{\n{arms}    }}\n}}\n\n\
         type Colour =\n    | Red\n    | Green\n    | Blue\n"
    )
}

/// How many instructions of `described` are ones `wanted` answers for.
fn counted(of: &Lowered, wanted: fn(&Instruction) -> bool) -> usize {
    body_of(of, "described")
        .instructions
        .iter()
        .filter(|instruction| wanted(instruction))
        .count()
}

/// Taking the matched value into a local, which a name that binds does and `_` does not.
fn is_a_store(instruction: &Instruction) -> bool {
    matches!(instruction, Instruction::Store { .. })
}

/// Falling to the next arm, which is what testing one alternative amounts to.
fn is_a_test(instruction: &Instruction) -> bool {
    matches!(instruction, Instruction::JumpIfFalse(_))
}

/// The answer both arms give, which is written once for each place the arms write it.
fn is_the_answer(instruction: &Instruction) -> bool {
    matches!(instruction, Instruction::Long(1))
}
