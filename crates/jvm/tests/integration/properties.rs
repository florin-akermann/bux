//! The invariants of `docs/specs/codegen.md`, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{
    Body, Class, ClassName, Descriptor, Instruction, Lowered, Method, MethodDescriptor,
};
use lumen_ir::{Label, Reached};

use crate::reader::read;

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
        assert_eq!(read.major, 71);
        assert_eq!(read.minor, 0);
    }
}

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
            },
        });
    }
    Lowered {
        classes: vec![class],
    }
}

fn leaving(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    instructions.push(Instruction::Return(Some(Descriptor::Long)));
    instructions
}
