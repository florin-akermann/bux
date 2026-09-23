//! What the frames of a written method say holds where branches meet, read back out of the bytes.
//!
//! `docs/specs/codegen.md` states which class a frame names where two branches meet.

use lumen_ir::{Class, ClassName, Descriptor, Extending, Instruction, Label, Lowered};

use crate::common::{body, module_with, one_of, taking};
use crate::reader::{ClassFile, TakeCode};

/// The module the branching method is written in, and the one whose type it gives back.
///
/// `demo` writes no class of `holder`, so only the program as a whole says what
/// `holder/Payment$Sent` and `holder/Payment$Pending` extend.
#[test]
fn where_variants_of_another_modules_type_meet_the_frame_names_the_base_that_module_declares() {
    let base = ClassName::new("holder/Payment");
    let sent = variant_of(&base, "Sent");
    let pending = variant_of(&base, "Pending");
    let chosen = either(&sent.name, &pending.name, &base);
    let mut declared = Class::new(base);
    declared.extending = Extending::ByItsVariants;
    let demo = lowered(vec![chosen]);
    let holder = lowered(vec![declared, sent, pending]);

    let files = lumen_jvm::write(&[demo, holder]);

    let met = stacks(&one_of(&files, "demo.class"), "chosen");
    assert_eq!(met.last(), Some(&vec!["holder/Payment".to_owned()]));
}

/// A module whose method `chosen` gives back its second parameter or its third, as `given`.
///
/// Its first parameter says which, so the two branches meet where the method gives one back.
fn either(first: &ClassName, second: &ClassName, given: &ClassName) -> Class {
    let reference = |class: &ClassName| Descriptor::Reference(class.clone());
    module_with(
        "chosen",
        taking(
            vec![Descriptor::Boolean, reference(first), reference(second)],
            Some(reference(given)),
        ),
        body(vec![
            Instruction::Load {
                slot: 0,
                of: Descriptor::Boolean,
            },
            Instruction::JumpIfFalse(Label(1)),
            Instruction::Load {
                slot: 1,
                of: reference(first),
            },
            Instruction::Jump(Label(2)),
            Instruction::Label(Label(1)),
            Instruction::Load {
                slot: 2,
                of: reference(second),
            },
            Instruction::Label(Label(2)),
            Instruction::Return(Some(reference(given))),
        ]),
    )
}

/// A variant of `base` named `name`, which extends the base and nothing else.
fn variant_of(base: &ClassName, name: &str) -> Class {
    let mut variant = Class::new(ClassName::new(&format!("{base}${name}")));
    variant.extends = base.clone();
    variant
}

/// One module of the program, holding `classes` and asking nothing of another.
fn lowered(classes: Vec<Class>) -> Lowered {
    Lowered {
        asks: lumen_ir::Asked::default(),
        classes,
    }
}

/// The classes each frame of the method `name` says are on the stack, frame by frame.
///
/// The writer writes every frame as a full frame, so each is read as one: a place, the locals,
/// then the stack, where only an object or an unbuilt instance carries two more bytes.
fn stacks(file: &ClassFile, name: &str) -> Vec<Vec<String>> {
    let table = file
        .method(name)
        .code
        .take_code()
        .frames
        .clone()
        .expect("a body that branches says what holds where");
    let mut frames = Frames {
        bytes: &table,
        at: 0,
    };
    let count = frames.u2();
    (0..count).map(|_| frames.stack(file)).collect()
}

/// The bytes of a `StackMapTable`, and how far into them a test has read.
struct Frames<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Frames<'_> {
    /// The stack of the next full frame, with its place and its locals read past.
    fn stack(&mut self, file: &ClassFile) -> Vec<String> {
        assert_eq!(self.u1(), 255, "every frame is written as a full frame");
        self.u2();
        let locals = self.u2();
        for _ in 0..locals {
            self.held(file);
        }
        let stacked = self.u2();
        (0..stacked).filter_map(|_| self.held(file)).collect()
    }

    /// The class one entry names, where it names one.
    fn held(&mut self, file: &ClassFile) -> Option<String> {
        match self.u1() {
            7 => Some(file.class(self.u2()).to_owned()),
            8 => {
                self.u2();
                None
            }
            _ => None,
        }
    }

    fn u1(&mut self) -> u8 {
        self.at += 1;
        self.bytes[self.at - 1]
    }

    fn u2(&mut self) -> u16 {
        self.at += 2;
        u16::from_be_bytes([self.bytes[self.at - 2], self.bytes[self.at - 1]])
    }
}
