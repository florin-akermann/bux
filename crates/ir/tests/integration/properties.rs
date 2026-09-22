//! The invariants of `docs/specs/codegen.md` that lowering holds, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{ClassName, Instruction, Lowered};

use crate::common;

/// The modules a property is checked over, each written in one construct or another.
const SOURCES: [&str; 10] = [
    "fn written() -> List<Int> {\n    [1, 2, 3]\n}\n",
    "fn held() -> Option<()> {\n    Some(())\n}\n",
    "fn answer() -> Int {\n    7\n}\n",
    "fn held(user: User) -> Int {\n    user.id\n}\n\ntype User = {\n    id: Int\n    active: Bool\n}\n",
    "fn told(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n",
    "fn walked(counts: List<Int>) -> Int {\n    var total = 0\n    for count in counts {\n        total += count\n    }\n    total\n}\n",
    "fn used() -> Result<Int, String> {\n    value := held()?\n    Ok(value + 1)\n}\n\nfn held() -> Result<Int, String> {\n    Ok(1)\n}\n",
    "fn wrapped() -> Option<Int> {\n    Some(identity(2))\n}\n\nfn identity<T>(value: T) -> T {\n    value\n}\n",
    "fn counted() -> Int {\n    2\n}\n\nfn main(arguments: List<String>) -> Int {\n    0\n}\n",
    "fn is_same(word: String, count: Int) -> Bool {\n    word == \"one\" && count != 2\n}\n",
];

/// The values a generated call passes first, each one a value of `Int`.
const FIRST: [&str; 3] = ["count", "1", "twice(count)"];

/// The values it passes after that, each one a value of `String`.
const REST: [&str; 2] = ["said", "\"hi\""];

/// A module calling `held` as `call` writes it, with what it calls declared below.
fn calling(call: &str) -> String {
    format!(
        "fn go(count: Int, said: String) -> String {{\n    {call}\n}}\n\n\
         fn held(count: Int, said: String) -> String {{\n    said\n}}\n\n\
         fn twice(count: Int) -> Int {{\n    count + count\n}}\n"
    )
}

/// `docs/specs/calls.md`: the two spellings of one call are one call, and lower as one.
#[hegel::test]
fn a_call_written_in_front_of_the_name_lowers_as_the_plain_call_does(tc: TestCase) {
    let first = tc.draw(gs::sampled_from(&FIRST));
    let rest = tc.draw(gs::sampled_from(&REST));

    let in_front = common::lowered(&calling(&format!("{first}.held({rest})")));
    let plainly = common::lowered(&calling(&format!("held({first}, {rest})")));

    assert_eq!(
        common::body_of(&in_front, "go").instructions,
        common::body_of(&plainly, "go").instructions
    );
}

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
        source.contains("fn main(arguments: List<String>) -> Int {"),
        "a program is a module that declares `main` at the one shape, and nothing else is"
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

/// How many elements a generated list is written with, which is what its array must be as long as.
const LENGTHS: [usize; 5] = [0, 1, 2, 3, 7];

#[hegel::test]
fn a_written_list_gathers_as_many_elements_as_it_writes_and_builds_one_list(tc: TestCase) {
    let many = tc.draw(gs::sampled_from(&LENGTHS));
    let elements: Vec<String> = (0..many).map(|at| at.to_string()).collect();
    let source = format!(
        "fn written() -> List<Int> {{\n    [{}]\n}}\n",
        elements.join(", ")
    );

    let lowered = common::lowered(&source);

    let body = common::body_of(&lowered, "written");
    let stored = body
        .instructions
        .iter()
        .filter(|step| **step == Instruction::StoreInArray)
        .count();
    let collected = body
        .instructions
        .iter()
        .filter(|step| **step == common::building_a_list())
        .count();
    assert_eq!(
        body.instructions.first(),
        Some(&Instruction::Integer(counting(many)))
    );
    assert_eq!(stored, many, "one store per element");
    assert_eq!(collected, 1, "one list however many elements it holds");
}

/// The length as the JVM counts it, which is what the array is made with.
fn counting(many: usize) -> i32 {
    i32::try_from(many).expect("a generated list is short")
}

#[hegel::test]
fn as_many_values_stand_for_nothing_as_there_are_units_reaching_a_reference(tc: TestCase) {
    let many = tc.draw(gs::sampled_from(&LENGTHS));
    let written: Vec<&str> = (0..many).map(|_| "()").collect();
    let source = format!(
        "fn written() -> List<()> {{\n    [{}]\n}}\n",
        written.join(", ")
    );

    let lowered = common::lowered(&source);

    let body = common::body_of(&lowered, "written");
    assert_eq!(
        standing_for_nothing(&body.instructions),
        many,
        "one value stands per `()` the list writes"
    );
}

/// How many values the instructions build to stand for a `()`, which is a bare `java.lang.Object`.
fn standing_for_nothing(instructions: &[Instruction]) -> usize {
    instructions
        .iter()
        .filter(|step| **step == Instruction::New(ClassName::new("java/lang/Object")))
        .count()
}
