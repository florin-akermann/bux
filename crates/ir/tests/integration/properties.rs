//! The invariants of `docs/specs/codegen.md` that lowering holds, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{Arithmetic, ClassName, Comparison, Descriptor, Instruction, Lowered};

use crate::common;

/// The modules a property is checked over, each written in one construct or another.
const SOURCES: [&str; 8] = [
    "fn answer() -> Int {\n    7\n}\n",
    "fn held(user: User) -> Int {\n    user.id\n}\n\ntype User = {\n    id: Int\n    active: Bool\n}\n",
    "fn told(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n",
    "fn walked(counts: List<Int>) -> Int {\n    var total = 0\n    for count in counts {\n        total += count\n    }\n    total\n}\n",
    "fn used() -> Result<Int, String> {\n    value := held()?\n    Ok(value + 1)\n}\n\nfn held() -> Result<Int, String> {\n    Ok(1)\n}\n",
    "fn wrapped() -> Option<Int> {\n    Some(identity(2))\n}\n\nfn identity<T>(value: T) -> T {\n    value\n}\n",
    "fn counted() -> Int {\n    2\n}\n\nfn main() -> () {\n    ()\n}\n",
    "fn same(word: String, count: Int) -> Bool {\n    word == \"one\" && count != 2\n}\n",
];

#[hegel::test]
fn lowering_one_module_twice_gives_the_same_classes(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    assert_eq!(common::lowered(source), common::lowered(source));
}

#[hegel::test]
fn every_class_a_module_writes_is_named_once(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    let lowered = common::lowered(source);

    let mut written = common::written(&lowered);
    let many = written.len();
    written.sort_unstable();
    written.dedup();
    assert_eq!(written.len(), many, "no two classes share a name");
}

#[hegel::test]
fn every_method_of_every_class_a_module_writes_ends_by_leaving_it(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    let lowered = common::lowered(source);

    for class in &lowered.classes {
        for method in &class.methods {
            let last = method.body.instructions.last();
            assert!(
                matches!(last, Some(lumen_ir::Instruction::Return(_))),
                "{}.{} ends with {last:?}",
                class.name,
                method.name
            );
        }
    }
}

#[hegel::test]
fn a_module_is_written_with_an_entry_point_exactly_when_it_declares_main(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    let lowered = common::lowered(source);

    assert_eq!(
        lumen_ir::is_a_program(&lowered),
        source.contains("fn main() -> () {"),
        "a program is a module that declares `main`, and nothing else is"
    );
}

/// The methods every JVM class inherits, none of which a Lumen class may declare.
const INHERITED: [&str; 9] = [
    "equals",
    "hashCode",
    "getClass",
    "toString",
    "clone",
    "finalize",
    "wait",
    "notify",
    "notifyAll",
];

#[hegel::test]
fn no_class_a_module_writes_declares_a_method_the_object_model_would_give_it(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    let lowered = common::lowered(source);

    for class in &lowered.classes {
        for method in &class.methods {
            assert!(
                !INHERITED.contains(&method.name.as_str()),
                "{} declares `{}`, which the object model gives it",
                class.name,
                method.name
            );
        }
    }
}

#[hegel::test]
fn the_one_equality_a_module_calls_is_the_one_string_declares(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    let lowered = common::lowered(source);

    for called in equalities_called(&lowered) {
        assert_eq!(
            called.written(),
            "java/lang/String",
            "`equals` on anything else answers by identity, which Lumen never asks about"
        );
    }
}

/// The class of every `equals` the module calls, whichever way the call is written.
fn equalities_called(lowered: &Lowered) -> Vec<ClassName> {
    lowered
        .classes
        .iter()
        .flat_map(|class| &class.methods)
        .flat_map(|method| &method.body.instructions)
        .filter_map(common::called)
        .filter(|reference| reference.name == "equals")
        .map(|reference| reference.class.clone())
        .collect()
}

/// The operators of `docs/specs/arithmetic.md` that have no answer for a zero divisor.
const DIVIDING: [&str; 2] = ["/", "%"];

/// What a division is written over, none of which says at compile time what the divisor is.
const OPERANDS: [&str; 3] = ["a", "b", "17"];

/// The ways a division is written, each holding it where `{}` is.
const CONTEXTS: [&str; 4] = [
    "fn go(a: Int, b: Int) -> Option<Int> {\n    {}\n}\n",
    "fn go(a: Int, b: Int) -> Int {\n    or({}, 0)\n}\n",
    "fn go(a: Int, b: Int) -> Option<Int> {\n    held := {}\n    held\n}\n",
    "fn go(a: Int, b: Int) -> Bool {\n    or({}, 0) == a\n}\n",
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
