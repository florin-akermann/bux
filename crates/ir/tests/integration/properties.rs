//! The invariants of `docs/specs/codegen.md` that lowering holds, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{Arithmetic, ClassName, Comparison, Descriptor, Instruction, Lowered};

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
    "fn counted() -> Int {\n    2\n}\n\nfn main() -> () {\n    ()\n}\n",
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

/// Every JVM class the library modules reach, which `docs/specs/io.md` names each of.
///
/// The last three are what the class file itself is made of rather than anything Bux writes: the
/// `Bool` an `Ok` carries is boxed, a guarded declaration asks whatever it caught what it says of
/// itself, and the arm of a `match` nothing reaches says so rather than runs on, which
/// `docs/specs/codegen.md` states.
const REACHED: [&str; 16] = [
    "java/lang/String",
    "java/lang/System",
    "java/io/PrintStream",
    "java/io/PrintWriter",
    "java/io/File",
    "java/nio/file/Path",
    "java/nio/file/Files",
    "java/nio/charset/Charset",
    "java/nio/charset/StandardCharsets",
    "java/util/stream/Stream",
    "java/util/List",
    "java/util/Iterator",
    "java/lang/Object",
    "java/lang/Boolean",
    "java/lang/Throwable",
    "java/lang/AssertionError",
];

/// The library modules that reach outside a program, which is what `docs/specs/io.md` is about.
const LIBRARY: [&str; 3] = ["environment", "files", "io"];

/// A call of each name the library modules declare, with the text it is given.
const CALLS: [&str; 8] = [
    "io.print(text)",
    "io.println(text)",
    "_ = files.read(text)",
    "_ = files.write(text, text)",
    "_ = files.listed(text)",
    "_ = files.made(text)",
    "_ = files.removed(text)",
    "_ = environment.read(text)",
];

/// Every way an `extern` reaches a member, each written with the result left to be filled in.
const REACHES: [(&str, &str); 4] = [
    (
        "extern field held() -> {result} = \"java.lang.System.out\"",
        "PrintStream",
    ),
    (
        "extern static worded(value: Int) -> {result} = \"java.lang.String.valueOf\"",
        "String",
    ),
    (
        "extern method trimmed(text: String) -> {result} = \"trim\"",
        "String",
    ),
    ("extern new named(path: String) -> {result}", "File"),
];

/// Every way a declaration wraps what the member gives back, written around the type it wraps.
const WRAPS: [&str; 3] = ["{held}", "Option<{held}>", "Result<{held}, String>"];

/// Every way an `extern` reaches a member giving a number, with the width left to be filled in.
const NUMBERS: [&str; 3] = [
    "extern field {width}held() -> {result} = \"java.lang.Integer.MAX_VALUE\"",
    "extern static {width}worded(text: String) -> {result} = \"java.lang.String.length\"",
    "extern method {width}trimmed(text: String) -> {result} = \"length\"",
];

/// What a declaration says its member's own descriptor gives back, which is written or is not.
const WIDTHS: [&str; 2] = ["", "int "];

/// Every way a declaration wraps a number, which is never an `Option` because none is `null`.
const AROUND: [&str; 2] = ["Int", "Result<Int, String>"];

#[hegel::test]
fn a_call_into_a_library_module_reaches_that_module_s_class_and_nothing_else(tc: TestCase) {
    let call = tc.draw(gs::sampled_from(&CALLS));
    let source = format!(
        "import environment\n\nimport files\n\nimport io\n\nfn go(text: String) -> () {{\n    {call}\n}}\n"
    );

    let reaching = LIBRARY
        .iter()
        .fold(lumen_types::Imported::default(), |imported, module| {
            imported.offering(module, common::library(module).offers())
        });
    let lowered = common::lowered_reaching(&source, &reaching);

    for reached in reached_by(&lowered, "go") {
        assert!(
            LIBRARY.contains(&reached.as_str()),
            "`{call}` reaches {reached}, and a call into a module reaches that module"
        );
    }
}

#[hegel::test]
fn every_class_a_library_module_reaches_is_one_the_spec_names(tc: TestCase) {
    let module = tc.draw(gs::sampled_from(&LIBRARY));

    let lowered = common::library(module).lowered();

    for class in &lowered.classes {
        for method in &class.methods {
            for reached in reached_in(&method.body) {
                assert!(
                    REACHED.contains(&reached.as_str()) || !reached.starts_with("java/"),
                    "{module}.{} reaches {reached}, which `docs/specs/io.md` does not name",
                    method.name
                );
            }
        }
    }
}

#[hegel::test]
fn a_declaration_guards_a_span_exactly_where_it_gives_back_a_result(tc: TestCase) {
    let (declared, held) = tc.draw(gs::sampled_from(&REACHES));
    let wrap = tc.draw(gs::sampled_from(&WRAPS));
    let result = wrap.replace("{held}", held);

    let lowered = common::lowered(&declaring(declared, &result));

    let body = common::body_of(&lowered, declared_name(declared));
    let guards = usize::from(result.starts_with("Result<"));
    assert_eq!(
        body.guards.len(),
        guards,
        "`{result}` guards {guards} spans"
    );
    for guard in &body.guards {
        assert_eq!(guard.catching, ClassName::new("java/lang/Throwable"));
    }
}

#[hegel::test]
fn a_result_leaves_an_ok_down_the_path_taken_and_an_err_down_the_one_thrown(tc: TestCase) {
    let (declared, held) = tc.draw(gs::sampled_from(&REACHES));
    let result = format!("Result<{held}, String>");

    let lowered = common::lowered(&declaring(declared, &result));

    let body = common::body_of(&lowered, declared_name(declared));
    let [guard] = body.guards.as_slice() else {
        panic!("a `Result` guards one span")
    };
    let (taken, thrown) = body.instructions.split_at(written_at(body, guard.handler));
    assert_eq!(built_by(taken), vec!["lumen/Result$Ok".to_owned()]);
    assert_eq!(built_by(thrown), vec!["lumen/Result$Err".to_owned()]);
}

#[hegel::test]
fn a_declaration_written_int_reaches_its_member_for_one_and_leaves_a_long_behind_it(tc: TestCase) {
    let declared = tc.draw(gs::sampled_from(&NUMBERS));
    let width = tc.draw(gs::sampled_from(&WIDTHS));
    let result = tc.draw(gs::sampled_from(&AROUND));
    let written = declared.replace("{width}", width);
    let widens = !width.is_empty();

    let lowered = common::lowered(&declaring(&written, result));

    let body = common::body_of(&lowered, declared_name(&written));
    let reached = if widens {
        Descriptor::Integer
    } else {
        Descriptor::Long
    };
    assert_eq!(
        gives_back(body),
        Some(reached),
        "`{written}` reaches its member for what it says that member gives back"
    );
    assert_eq!(
        body.instructions.contains(&Instruction::Widen),
        widens,
        "an `int` is widened to the `Int` declared, and what is already a `long` is not"
    );
}

/// Which kind of class the type a generated `method` is called on is, written or not written.
const CLASSES: [&str; 2] = ["", "interface "];

#[hegel::test]
fn a_method_on_a_type_written_interface_is_called_the_way_the_jvm_calls_an_interface_s(
    tc: TestCase,
) {
    let class = tc.draw(gs::sampled_from(&CLASSES));
    let an_interface = !class.is_empty();
    let declared = "extern method to_path(file: File) -> File = \"toPath\"";
    let source = format!("{declared}\n\nextern type {class}File = \"java.io.File\"\n");

    let lowered = common::lowered(&source);

    let body = common::body_of(&lowered, "to_path");
    let called = body
        .instructions
        .iter()
        .find(|instruction| common::called(instruction).is_some())
        .expect("a `method` calls the member it names");
    assert_eq!(
        matches!(called, Instruction::InvokeInterface(_)),
        an_interface,
        "`extern type {class}File` is called as {}",
        if an_interface {
            "an interface"
        } else {
            "a class"
        }
    );
}
/// What the member `body` reaches gives back, which is the field it reads or the method it calls.
fn gives_back(body: &lumen_ir::Body) -> Option<Descriptor> {
    body.instructions
        .iter()
        .find_map(|instruction| match instruction {
            Instruction::GetStatic(field) => Some(field.of.clone()),
            other => common::called(other).and_then(|called| called.descriptor.result.clone()),
        })
}

/// A module holding `declared` with `result` filled in, over the types that declaration names.
fn declaring(declared: &str, result: &str) -> String {
    format!(
        "{}\n\nextern type PrintStream = \"java.io.PrintStream\"\n\nextern type File = \"java.io.File\"\n",
        declared.replace("{result}", result)
    )
}

/// The name `declared` declares, which is the method the module writes it as.
fn declared_name(declared: &str) -> &str {
    declared
        .split_whitespace()
        .find_map(|written| written.split_once('('))
        .map(|(name, _)| name)
        .expect("a declaration writes its name in front of the parameters it takes")
}

/// Every class the method `name` of the module reaches, by a call or by a field.
fn reached_by(lowered: &Lowered, name: &str) -> Vec<String> {
    reached_in(common::body_of(lowered, name))
}

/// Every class `body` reaches, by a call or by a field.
fn reached_in(body: &lumen_ir::Body) -> Vec<String> {
    body.instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::New(class) | Instruction::Cast(class) => Some(class.written().to_owned()),
            Instruction::GetStatic(field) => Some(field.class.written().to_owned()),
            other => common::called(other).map(|called| called.class.written().to_owned()),
        })
        .collect()
}

/// Where `label` is written among `body`'s instructions.
fn written_at(body: &lumen_ir::Body, label: lumen_ir::Label) -> usize {
    body.instructions
        .iter()
        .position(|instruction| instruction == &Instruction::Label(label))
        .unwrap_or_else(|| panic!("{label:?} is written in the body"))
}

/// The answers `instructions` builds, in the order they are built.
///
/// A declaration may build the class it gives back on the way, which is not an answer and is not
/// one of these.
fn built_by(instructions: &[Instruction]) -> Vec<String> {
    instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::New(class) => Some(class.written().to_owned()),
            _ => None,
        })
        .filter(|built| built.starts_with("lumen/Result$"))
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
        .filter(|step| **step == Instruction::CollectList)
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
