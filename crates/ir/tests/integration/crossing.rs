//! The invariants of a declaration that reaches a JVM member, on generated input.
//!
//! `docs/specs/interop.md` states what an `extern` may say and what each spelling reaches,
//! and `docs/specs/io.md` names every class the library modules reach.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{ClassName, Descriptor, Instruction, Lowered};

use crate::common;

/// Every JVM class the library modules reach, which `docs/specs/io.md` names each of.
///
/// The last three are what the class file itself is made of rather than anything Bux writes: the
/// `Bool` an `Ok` carries is boxed, a guarded declaration asks whatever it caught what it says of
/// itself, and the arm of a `match` nothing reaches says so rather than runs on, which
/// `docs/specs/codegen.md` states.
const REACHED: [&str; 18] = [
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
    "java/lang/Character",
    "java/lang/Long",
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
const WIDTHS: [&str; 3] = ["", "int ", "char "];

/// Every way a declaration wraps a number, which is never an `Option` because none is `null`.
const AROUND: [&str; 2] = ["Int", "Result<Int, String>"];

/// Every way an `extern` reaches a member that takes a number, with the narrowing left open.
const TAKING: [&str; 2] = [
    "extern static worded({taken}index: Int) -> {result} = \"java.lang.String.valueOf\"",
    "extern method charred(text: String, {taken}index: Int) -> {result} = \"substring\"",
];

/// What a declaration says its member's own descriptor takes, which is written or is not.
const NARROWING: [&str; 2] = ["", "int "];

/// Every way a declaration that narrows an argument wraps what its member gives back.
const ANSWERING: [&str; 2] = ["Option<String>", "Result<Option<String>, String>"];

/// Where an argument outside the `int` range lands, which the lowering writes only where one may.
const UNFIT: lumen_ir::Label = lumen_ir::Label(4);

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
            for reached in reached_in(&method.body.instructions) {
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
    let (taken, thrown) = body.instructions.split_at(
        landed_at(body, guard.handler).expect("a guard writes the label its handler begins at"),
    );
    assert_eq!(built_by(taken), vec!["lumen/Result$Ok".to_owned()]);
    assert_eq!(built_by(thrown), vec!["lumen/Result$Err".to_owned()]);
}

#[hegel::test]
fn a_declaration_written_with_a_width_reaches_its_member_for_one_and_leaves_a_long(tc: TestCase) {
    let declared = tc.draw(gs::sampled_from(&NUMBERS));
    let width = tc.draw(gs::sampled_from(&WIDTHS));
    let result = tc.draw(gs::sampled_from(&AROUND));
    let written = declared.replace("{width}", width);
    let widens = !width.is_empty();
    let gives = match width.trim() {
        "int" => Descriptor::Integer,
        "char" => Descriptor::Character,
        _ => Descriptor::Long,
    };

    let lowered = common::lowered(&declaring(&written, result));

    let body = common::body_of(&lowered, declared_name(&written));
    assert_eq!(
        gives_back(body),
        Some(gives),
        "`{written}` reaches its member for what it says that member gives back"
    );
    assert_eq!(
        body.instructions.contains(&Instruction::Widen),
        widens,
        "a width is widened to the `Int` declared, and what is already a `long` is not"
    );
}

#[hegel::test]
fn a_narrowed_argument_outside_the_int_range_answers_none_and_reaches_no_member(tc: TestCase) {
    let declared = tc.draw(gs::sampled_from(&TAKING));
    let taken = tc.draw(gs::sampled_from(&NARROWING));
    let result = tc.draw(gs::sampled_from(&ANSWERING));
    let written = declared.replace("{taken}", taken);
    let narrows = !taken.is_empty();

    let lowered = common::lowered(&declaring(&written, result));

    let body = common::body_of(&lowered, declared_name(&written));
    assert_eq!(
        body.instructions.contains(&Instruction::Narrow),
        narrows,
        "`{written}` narrows the argument its member takes an `int` for, and no other"
    );
    let Some(unfit) = landed_at(body, UNFIT) else {
        assert!(!narrows, "a narrowed argument has a `None` to land on");
        return;
    };
    assert!(narrows, "nothing lands there where no argument is narrowed");
    let reached = reached_in(answered_from(body, unfit));
    assert_eq!(
        reached
            .iter()
            .filter(|class| !class.starts_with(BUILT))
            .count(),
        0,
        "an argument outside the `int` range reaches the member not at all, and reached {reached:?}"
    );
}

/// The path from `from` to the answer it gives back, which is where an unfit argument goes.
fn answered_from(body: &lumen_ir::Body, from: usize) -> &[Instruction] {
    let path = &body.instructions[from..];
    let answered = path
        .iter()
        .position(|instruction| matches!(instruction, Instruction::Return(_)))
        .expect("every path out of a body gives an answer back");
    &path[..=answered]
}

/// Where `label` is written among `body`'s instructions, where the lowering wrote one.
fn landed_at(body: &lumen_ir::Body, label: lumen_ir::Label) -> Option<usize> {
    body.instructions
        .iter()
        .position(|instruction| instruction == &Instruction::Label(label))
}

/// What the name of every class the compiler writes begins with, Java's own carrying no such thing.
const BUILT: &str = "lumen/";

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
    reached_in(&common::body_of(lowered, name).instructions)
}

/// Every class `instructions` reaches, by a call or by a field.
fn reached_in(instructions: &[Instruction]) -> Vec<String> {
    instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::New(class) | Instruction::Cast(class) => Some(class.written().to_owned()),
            Instruction::GetStatic(field) => Some(field.class.written().to_owned()),
            other => common::called(other).map(|called| called.class.written().to_owned()),
        })
        .collect()
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
