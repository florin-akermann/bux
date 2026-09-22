//! The invariants of a lowered body that takes one path or another, on generated input.
//!
//! A division tests its divisor, an `or` works its fallback out, and a `?` hands a `None`
//! back: each writes two paths, and `docs/specs/codegen.md` states what each path holds.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{Arithmetic, ClassName, Comparison, Descriptor, Instruction, Lowered};

use crate::common;

/// The operators of `docs/specs/arithmetic.md` that have no answer for a zero divisor.
const DIVIDING: [&str; 2] = ["/", "%"];

/// What a division is written over, none of which says at compile time what the divisor is.
const OPERANDS: [&str; 3] = ["a", "b", "17"];

/// The ways a division is written, each holding it where `{}` is.
const CONTEXTS: [&str; 4] = [
    "fn go(a: Int, b: Int) -> Option<Int> {\n    {}\n}\n",
    "fn go(a: Int, b: Int) -> Int {\n    or({}, 0)\n}\n",
    "fn go(a: Int, b: Int) -> Option<Int> {\n    held := {}\n    held\n}\n",
    "fn is_same(a: Int, b: Int) -> Bool {\n    or({}, 0) == a\n}\n",
];

/// One module dividing, drawn from the operators, the operands, and where it is written.
fn dividing(tc: &mut TestCase) -> String {
    let operator = tc.draw(gs::sampled_from(&DIVIDING));
    let dividend = tc.draw(gs::sampled_from(&OPERANDS));
    let divisor = tc.draw(gs::sampled_from(&OPERANDS));
    let context = tc.draw(gs::sampled_from(&CONTEXTS));
    context.replace("{}", &format!("{dividend} {operator} {divisor}"))
}

#[hegel::test]
fn every_division_tests_its_divisor_before_it_divides(mut tc: TestCase) {
    let source = dividing(&mut tc);

    let lowered = common::lowered(&source);

    for method in methods_of(&lowered) {
        for (at, divisor) in divisors_of(method) {
            assert!(
                method[..at]
                    .windows(3)
                    .any(|window| window == tested_against_zero(divisor)),
                "{source} divides by slot {divisor} without testing that slot against zero"
            );
        }
    }
}

/// Where each division is, and the slot it divides by, which the load before it names.
fn divisors_of(method: &[Instruction]) -> Vec<(usize, u16)> {
    method
        .iter()
        .enumerate()
        .filter(|(_, instruction)| {
            matches!(
                instruction,
                Instruction::Arithmetic(Arithmetic::Divide | Arithmetic::Remainder)
            )
        })
        .map(|(at, _)| match method.get(at.wrapping_sub(1)) {
            Some(Instruction::Load { slot, .. }) => (at, *slot),
            other => panic!("a division divides what was loaded for it, not {other:?}"),
        })
        .collect()
}

/// The three instructions that ask whether the slot a division divides by holds zero.
fn tested_against_zero(divisor: u16) -> [Instruction; 3] {
    [
        Instruction::Load {
            slot: divisor,
            of: Descriptor::Long,
        },
        Instruction::Long(0),
        Instruction::CompareLongs(Comparison::Equal),
    ]
}

#[hegel::test]
fn every_division_builds_both_answers_a_divisor_can_have(mut tc: TestCase) {
    let source = dividing(&mut tc);

    let lowered = common::lowered(&source);

    for variant in ["lumen/Option$Some", "lumen/Option$None"] {
        let built = ClassName::new(variant);
        assert!(
            methods_of(&lowered).any(|method| method.contains(&Instruction::New(built.clone()))),
            "{source} builds a {variant}"
        );
    }
}

#[hegel::test]
fn or_is_written_out_where_it_is_used_rather_than_called(tc: TestCase) {
    let fallback = tc.draw(gs::sampled_from(&OPERANDS));
    let source = format!("fn go(a: Int, b: Int) -> Int {{\n    or(a / b, {fallback})\n}}\n");

    let lowered = common::lowered(&source);
    let body = &common::body_of(&lowered, "go").instructions;

    assert!(
        !body
            .iter()
            .filter_map(common::called)
            .any(|reference| reference.name == "or"),
        "version 0.1 has no class to call `or` on, so it is written where it is used"
    );
    assert!(
        body.contains(&Instruction::CompareIntegers(Comparison::Equal)),
        "`or` asks which variant it was handed"
    );
    assert!(
        body.iter().any(|instruction| matches!(
            instruction,
            Instruction::GetField(field) if field.name == "value0"
        )),
        "`or` reads what a `Some` holds"
    );
}

#[hegel::test]
fn or_works_out_its_fallback_before_it_asks_which_variant_it_was_handed(tc: TestCase) {
    let operator = tc.draw(gs::sampled_from(&["+", "-", "*"]));
    let source = format!("fn go(a: Int, b: Int) -> Int {{\n    or(a / b, a {operator} b)\n}}\n");

    let lowered = common::lowered(&source);
    let body = &common::body_of(&lowered, "go").instructions;

    let worked_out = body
        .iter()
        .position(|instruction| instruction == &Instruction::Arithmetic(counted(operator)))
        .expect("the fallback is worked out");
    let asked = body
        .iter()
        .position(|instruction| instruction == &Instruction::CompareIntegers(Comparison::Equal))
        .expect("`or` asks which variant it was handed");

    assert!(
        worked_out < asked,
        "a call works out its arguments, and `or` is a call: {source}"
    );
}

/// The class an `Option` is held as, which is what a `?` on one hands back.
const OPTION: &str = "lumen/Option";

/// The tag a `None` carries, which is its position in the declaration of `Option`.
const NONE: i32 = 1;

/// The ways a `?` on an `Option` is written, each holding a division where `{}` is.
const PROPAGATING: [&str; 5] = [
    "fn go(a: Int, b: Int) -> Option<Int> {\n    Some(({})?)\n}\n",
    "fn go(a: Int, b: Int) -> Option<Int> {\n    held := ({})?\n    Some(held)\n}\n",
    "fn go(a: Int, b: Int) -> Option<Int> {\n    Some((({})? / 2)? + 2 * 5)\n}\n",
    "fn is_same(a: Int, b: Int) -> Option<Bool> {\n    Some(({})? == a)\n}\n",
    "fn go(a: Int, b: Int) -> Option<Int> {\n    if a == b {\n        return Some(({})?)\n    }\n    Some(0)\n}\n",
];

#[hegel::test]
fn every_question_mark_on_an_option_gives_the_none_it_was_handed_back_unchanged(tc: TestCase) {
    let dividend = tc.draw(gs::sampled_from(&OPERANDS));
    let divisor = tc.draw(gs::sampled_from(&OPERANDS));
    let context = tc.draw(gs::sampled_from(&PROPAGATING));
    let source = context.replace("{}", &format!("{dividend} / {divisor}"));

    let lowered = common::lowered(&source);

    let mut propagated = 0;
    for method in methods_of(&lowered) {
        for at in asked_which_variant(method) {
            propagated += 1;
            assert_eq!(
                &method[at..at + 2],
                handed_back(held_in(method, at)),
                "{source} builds a `None` rather than handing back the one it was given"
            );
        }
    }
    assert!(propagated > 0, "{source} writes a `?` on an `Option`");
}

/// Where each `?` on an `Option` jumps past the case it hands back, which is what follows.
fn asked_which_variant(method: &[Instruction]) -> Vec<usize> {
    method
        .windows(4)
        .enumerate()
        .filter(|(_, window)| tests_the_none_tag(window))
        .map(|(at, _)| at + 4)
        .collect()
}

/// Whether these four instructions ask whether an `Option` is the `None` a `?` hands back.
fn tests_the_none_tag(window: &[Instruction]) -> bool {
    matches!(&window[0], Instruction::GetField(field)
        if field.class == ClassName::new(OPTION) && field.name == "tag")
        && window[1] == Instruction::Integer(NONE)
        && window[2] == Instruction::CompareIntegers(Comparison::Equal)
        && matches!(window[3], Instruction::JumpIfFalse(_))
}

/// The slot the `Option` a `?` at `at` was handed is set aside in, which the test read it from.
fn held_in(method: &[Instruction], at: usize) -> u16 {
    match &method[at - 5] {
        Instruction::Load { slot, .. } => *slot,
        other => panic!("a `?` reads the tag of what it loaded, not of {other:?}"),
    }
}

/// Loading `slot` and giving it back, which is the whole of the case a `?` propagates.
fn handed_back(slot: u16) -> [Instruction; 2] {
    let of = Descriptor::reference(OPTION);
    [
        Instruction::Load {
            slot,
            of: of.clone(),
        },
        Instruction::Return(Some(of)),
    ]
}

/// The instruction an operator written in a fallback lowers to.
fn counted(operator: &str) -> Arithmetic {
    match operator {
        "+" => Arithmetic::Add,
        "-" => Arithmetic::Subtract,
        _ => Arithmetic::Multiply,
    }
}

/// Every method of every class the module writes, as the instructions each is made of.
fn methods_of(lowered: &Lowered) -> impl Iterator<Item = &Vec<Instruction>> {
    lowered
        .classes
        .iter()
        .flat_map(|class| &class.methods)
        .map(|method| &method.body.instructions)
}
