//! The invariants of `docs/specs/codegen.md` that lowering holds, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{ClassName, Lowered};

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
