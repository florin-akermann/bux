//! The invariants of `docs/specs/codegen.md`, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{
    Body, Class, ClassName, Descriptor, Instruction, Lowered, Method, MethodDescriptor,
};
use lumen_ir::{Label, Reached};

use crate::common;
use crate::reader::{ClassFile, Constant, read};

/// The modules the whole-compiler properties are checked over.
const SOURCES: [&str; 8] = [
    "fn answer() -> Int {\n    7\n}\n",
    "fn held(user: User) -> Int {\n    user.id\n}\n\ntype User = {\n    id: Int\n}\n",
    "fn told(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n",
    "fn walked(counts: List<Int>) -> Int {\n    var total = 0\n    for count in counts {\n        total += count\n    }\n    total\n}\n",
    "fn used() -> Result<Int, String> {\n    value := held()?\n    Ok(value + 1)\n}\n\nfn held() -> Result<Int, String> {\n    Ok(1)\n}\n",
    "fn number(user: User) -> Int {\n    user.home.number\n}\n\ntype User = {\n    id: Int\n    home: Address\n}\n\ntype Address = {\n    number: Int\n}\n",
    "fn kept(count: Int) -> Int {\n    identity(count)\n}\n\nfn is_kept(flag: Bool) -> Bool {\n    identity(flag)\n}\n\nfn identity<T>(value: T) -> T {\n    value\n}\n",
    "fn kept(count: Int) -> Int {\n    through(count)\n}\n\nfn through<T>(value: T) -> T {\n    identity(value)\n}\n\nfn identity<T>(value: T) -> T {\n    value\n}\n",
];

#[hegel::test]
fn compiling_one_source_twice_gives_byte_identical_class_files(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    assert_eq!(common::compiled(source), common::compiled(source));
}

#[hegel::test]
fn every_class_a_module_compiles_to_begins_with_the_magic_and_the_current_version(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    for file in common::compiled(source) {
        let written = read(&file.bytes);
        assert_eq!(written.magic, 0xCAFE_BABE, "{}", file.path);
        assert_eq!(written.major, 72);
        assert_eq!(written.minor, 65535);
    }
}

#[hegel::test]
fn every_constant_a_compiled_class_names_is_one_its_own_pool_holds(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    for file in common::compiled(source) {
        let written = read(&file.bytes);
        let held = written.pool.len();
        assert!(written.this as usize <= held, "{}", file.path);
        assert!(written.extends as usize <= held);
    }
}

#[hegel::test]
fn every_descriptor_a_class_asks_to_load_first_names_another_class_the_build_writes(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    let files = common::compiled(source);
    let written: Vec<String> = files
        .iter()
        .map(|file| format!("L{};", file.path.trim_end_matches(".class")))
        .collect();
    for (file, itself) in files.iter().zip(&written) {
        for wanted in read(&file.bytes).loadable {
            assert!(written.contains(&wanted), "{} asks for {wanted}", file.path);
            assert_ne!(&wanted, itself, "{} asks for itself", file.path);
        }
    }
}

/// Which body a generated method is built from.
const BODIES: [usize; 4] = [0, 1, 2, 3];

/// The bodies a generated method is built from, each leaving a whole number behind.
fn body(which: usize) -> Vec<Instruction> {
    match which {
        0 => vec![Instruction::Long(0)],
        1 => vec![Instruction::Long(-9_000_000_000)],
        2 => vec![
            Instruction::Long(2),
            Instruction::Long(3),
            Instruction::Arithmetic(lumen_ir::Arithmetic::Add),
        ],
        _ => vec![
            Instruction::Boolean(true),
            Instruction::JumpIfFalse(Label(0)),
            Instruction::Long(1),
            Instruction::Return(Some(Descriptor::Long)),
            Instruction::Label(Label(0)),
            Instruction::Long(2),
        ],
    }
}

#[hegel::test]
fn writing_a_class_twice_gives_the_same_bytes(tc: TestCase) {
    let lowered = a_module(&tc);

    assert_eq!(lumen_jvm::write(&lowered), lumen_jvm::write(&lowered));
}

#[hegel::test]
fn every_class_written_begins_with_the_magic_and_the_current_version(tc: TestCase) {
    let lowered = a_module(&tc);

    for file in lumen_jvm::write(&lowered) {
        let read = read(&file.bytes);
        assert_eq!(read.magic, 0xCAFE_BABE);
        assert_eq!(read.major, 72);
        assert_eq!(read.minor, 65535);
    }
}

#[hegel::test]
fn no_method_written_for_a_function_a_module_declares_names_object(tc: TestCase) {
    let source = tc.draw(gs::sampled_from(&SOURCES));

    let module = read(&common::compiled(source)[0].bytes);

    for method in &module.methods {
        assert!(
            !method.descriptor.contains("java/lang/Object"),
            "{} is written {}",
            method.name,
            method.descriptor
        );
    }
}

#[hegel::test]
fn a_generic_used_at_a_whole_number_is_written_taking_and_giving_back_a_long(tc: TestCase) {
    let through = tc.draw(gs::sampled_from(&GENERICS));
    let source =
        format!("fn kept(count: Int) -> Int {{\n    {through}(count)\n}}\n{WRITTEN_OVER_ONE_TYPE}");

    let module = read(&common::compiled(&source)[0].bytes);

    let written = module
        .methods
        .iter()
        .find(|method| method.name == format!("{through}$Int"))
        .unwrap_or_else(|| panic!("{source} writes {through} at `Int`"));
    assert_eq!(written.descriptor, "(J)J");
    assert!(
        !names(&module).any(|held| held == "java/lang/Long"),
        "a whole number crossing a generic is a `long`, so nothing boxes it"
    );
}

/// Every name the class holds in its pool, which is every class and member it can reach.
fn names(module: &ClassFile) -> impl Iterator<Item = &str> {
    module.pool.iter().filter_map(|constant| match constant {
        Constant::Utf8(text) => Some(text.as_str()),
        _ => None,
    })
}

/// The generic functions the specialization property calls, each giving back what it was given.
const GENERICS: [&str; 2] = ["identity", "through"];

/// One generic that calls another, so a use of either settles both.
const WRITTEN_OVER_ONE_TYPE: &str = concat!(
    "\nfn through<T>(value: T) -> T {\n    identity(value)\n}\n\n",
    "fn identity<T>(value: T) -> T {\n    value\n}\n"
);

#[hegel::test]
fn every_method_a_module_declares_is_written_as_a_method_of_it(tc: TestCase) {
    let lowered = a_module(&tc);
    let declared: Vec<&str> = lowered.classes[0]
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect();

    let written = read(&lumen_jvm::write(&lowered)[0].bytes);

    let names: Vec<&str> = written
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect();
    assert_eq!(names, declared);
}

#[hegel::test]
fn every_constant_a_class_names_is_one_the_pool_holds(tc: TestCase) {
    let lowered = a_module(&tc);

    let written = read(&lumen_jvm::write(&lowered)[0].bytes);

    assert!(written.this as usize <= written.pool.len());
    assert!(written.extends as usize <= written.pool.len());
    for method in &written.methods {
        assert!(!method.name.is_empty());
    }
}

/// A module of one to four methods, each built from one of the bodies above.
fn a_module(tc: &TestCase) -> Lowered {
    let mut chosen = tc.draw(gs::vecs(gs::sampled_from(&BODIES)).max_size(4));
    chosen.push(0);
    let mut class = Class::new(ClassName::new("demo"));
    for (at, which) in chosen.iter().enumerate() {
        class.methods.push(Method {
            name: format!("answer{at}"),
            descriptor: MethodDescriptor::new(Vec::new(), Some(Descriptor::Long)),
            reached: Reached::ThroughTheClass,
            body: Body {
                instructions: leaving(body(*which)),
                locals: 0,
                guards: Vec::new(),
            },
        });
    }
    Lowered {
        asks: lumen_ir::Asked::default(),
        classes: vec![class],
    }
}

fn leaving(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    instructions.push(Instruction::Return(Some(Descriptor::Long)));
    instructions
}
