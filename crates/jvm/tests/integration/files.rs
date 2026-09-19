//! What the writer writes, read back with a reader that is not the writer run backwards.
//!
//! `docs/specs/codegen.md` states each of these.

use lumen_ir::{Class, ClassName, Comparison, Descriptor, Extending, Field, Instruction};
use lumen_ir::{Label, Lowered, Method, Reached};

use crate::common::{body, module_with, taking, written};
use crate::reader::TakeCode;

#[test]
fn a_class_file_begins_with_the_magic_and_the_version_of_jdk_28_marked_preview() {
    let class = module_with("nothing", taking(Vec::new(), None), body(vec![]));

    let file = written(class);

    assert_eq!(file.magic, 0xCAFE_BABE);
    assert_eq!(file.major, 72, "JDK 28");
    assert_eq!(file.minor, 65535, "preview, which value classes are there");
}

#[test]
fn a_class_is_written_without_the_identity_bit_which_is_what_makes_it_a_value_class() {
    let mut base = Class::new(ClassName::new("demo/Payment"));
    base.extending = Extending::ByItsVariants;

    let closed = written(Class::new(ClassName::new("demo/User")));
    let open = written(base);

    assert_eq!(closed.access & 0x0020, 0, "a record has no identity");
    assert_eq!(open.access & 0x0020, 0, "nor has the base of a type");
}

#[test]
fn a_class_names_itself_and_what_it_extends() {
    let class = module_with("nothing", taking(Vec::new(), None), body(vec![]));

    let file = written(class);

    assert_eq!(file.class(file.this), "demo");
    assert_eq!(file.class(file.extends), "java/lang/Object");
}

#[test]
fn a_class_nothing_extends_is_written_final_and_one_its_variants_extend_is_written_abstract() {
    let mut base = Class::new(ClassName::new("demo/Payment"));
    base.extending = Extending::ByItsVariants;

    let closed = written(module_with(
        "nothing",
        taking(Vec::new(), None),
        body(vec![]),
    ));
    let open = written(base);

    assert_eq!(closed.access & 0x0010, 0x0010, "final");
    assert_eq!(open.access & 0x0400, 0x0400, "abstract");
}

#[test]
fn a_function_of_the_module_is_a_public_static_method_named_as_it_is_written() {
    let class = module_with(
        "answer",
        taking(Vec::new(), Some(Descriptor::Long)),
        body(vec![
            Instruction::Long(42),
            Instruction::Return(Some(Descriptor::Long)),
        ]),
    );

    let file = written(class);
    let method = file.method("answer");

    assert_eq!(method.descriptor, "()J");
    assert_eq!(method.access & 0x0008, 0x0008, "static");
    assert_eq!(method.access & 0x0001, 0x0001, "public");
}

#[test]
fn a_method_says_how_deep_its_stack_goes_and_how_many_locals_it_uses() {
    let class = module_with(
        "add",
        taking(
            vec![Descriptor::Long, Descriptor::Long],
            Some(Descriptor::Long),
        ),
        body(vec![
            Instruction::Load {
                slot: 0,
                of: Descriptor::Long,
            },
            Instruction::Load {
                slot: 2,
                of: Descriptor::Long,
            },
            Instruction::Arithmetic(lumen_ir::Arithmetic::Add),
            Instruction::Return(Some(Descriptor::Long)),
        ]),
    );

    let file = written(class);
    let code = file.method("add").code.take_code();

    assert_eq!(code.max_stack, 4, "two whole numbers, each two words wide");
    assert_eq!(code.max_locals, 4);
}

#[test]
fn a_field_of_a_class_is_written_public_and_final() {
    let mut class = Class::new(ClassName::new("demo/User"));
    class.fields.push(Field {
        name: "id".to_owned(),
        of: Descriptor::Long,
    });

    let file = written(class);

    assert_eq!(file.fields.len(), 1);
    assert_eq!(file.fields[0].name, "id");
    assert_eq!(file.fields[0].descriptor, "J");
    assert_eq!(file.fields[0].access & 0x0010, 0x0010, "final");
    assert_eq!(
        file.fields[0].access & 0x0800,
        0x0800,
        "strict: written before the constructor hands itself up"
    );
}

#[test]
fn a_body_that_never_branches_carries_no_stack_map() {
    let class = module_with(
        "answer",
        taking(Vec::new(), Some(Descriptor::Long)),
        body(vec![
            Instruction::Long(1),
            Instruction::Return(Some(Descriptor::Long)),
        ]),
    );

    let file = written(class);
    let code = file.method("answer").code.take_code();

    assert!(code.frames.is_none());
}

#[test]
fn a_body_that_branches_carries_a_stack_map_with_one_entry_for_each_place_a_jump_lands() {
    let class = module_with(
        "larger",
        taking(
            vec![Descriptor::Long, Descriptor::Long],
            Some(Descriptor::Long),
        ),
        body(vec![
            Instruction::Load {
                slot: 0,
                of: Descriptor::Long,
            },
            Instruction::Load {
                slot: 2,
                of: Descriptor::Long,
            },
            Instruction::CompareLongs(Comparison::Greater),
            Instruction::JumpIfFalse(Label(1)),
            Instruction::Load {
                slot: 0,
                of: Descriptor::Long,
            },
            Instruction::Return(Some(Descriptor::Long)),
            Instruction::Label(Label(1)),
            Instruction::Load {
                slot: 2,
                of: Descriptor::Long,
            },
            Instruction::Return(Some(Descriptor::Long)),
        ]),
    );

    let file = written(class);
    let code = file.method("larger").code.take_code();
    let frames = code
        .frames
        .as_ref()
        .expect("a body that branches says what holds where");

    // Two of the assembler's own, for the comparison, and the one the body wrote.
    assert_eq!(u16::from_be_bytes([frames[0], frames[1]]), 3);
}

#[test]
fn every_class_of_a_lowered_program_is_written_to_the_path_its_name_gives() {
    let lowered = Lowered {
        classes: vec![
            Class::new(ClassName::new("demo")),
            Class::new(ClassName::new("demo/User")),
            Class::new(ClassName::new("lumen/Option")),
        ],
    };

    let written = lumen_jvm::write(&lowered);

    let paths: Vec<&str> = written.iter().map(|file| file.path.as_str()).collect();
    assert_eq!(
        paths,
        ["demo.class", "demo/User.class", "lumen/Option.class"]
    );
}

#[test]
fn a_constructor_is_reached_through_an_instance_rather_than_through_the_class() {
    let mut class = Class::new(ClassName::new("demo/User"));
    class.methods.push(Method {
        name: "<init>".to_owned(),
        descriptor: taking(Vec::new(), None),
        reached: Reached::ThroughAnInstance,
        body: body(vec![Instruction::Return(None)]),
    });

    let file = written(class);

    assert_eq!(file.method("<init>").access & 0x0008, 0, "not static");
}

#[test]
fn a_whole_number_is_pushed_from_the_pool_and_given_back_as_one() {
    let class = module_with(
        "answer",
        taking(Vec::new(), Some(Descriptor::Long)),
        body(vec![
            Instruction::Long(42),
            Instruction::Return(Some(Descriptor::Long)),
        ]),
    );

    let file = written(class);
    let code = file.method("answer").code.take_code();

    assert_eq!(code.instructions[0], 0x14, "ldc2_w");
    assert_eq!(code.instructions[3], 0xAD, "lreturn");
    assert_eq!(code.instructions.len(), 4);
}

#[test]
fn a_small_whole_number_is_pushed_without_reaching_the_pool() {
    let class = module_with(
        "tag",
        taking(Vec::new(), Some(Descriptor::Integer)),
        body(vec![
            Instruction::Integer(3),
            Instruction::Return(Some(Descriptor::Integer)),
        ]),
    );

    let file = written(class);
    let code = file.method("tag").code.take_code();

    assert_eq!(code.instructions, [0x10, 3, 0xAC], "bipush 3, ireturn");
}
