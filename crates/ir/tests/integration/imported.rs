//! What a module writes for a type another module declares, which is nothing but the uses of it.
//!
//! `docs/specs/modules.md` states that the type is the declaring module's own, so the classes
//! stay that module's too: this one reaches them by name and writes none of them again.

use lumen_ir::{ClassName, Descriptor, Instruction};

use crate::common;

/// A module importing `held` and writing `body` as the body of the one function it declares.
fn reaching(signature: &str, body: &str) -> String {
    format!("import held\n\nfn go({signature}) {{\n    {body}\n}}\n")
}

/// Asserts that `body` builds a value of `built`, which is a class the other module writes.
fn builds(body: &str, built: &str) {
    let lowered = common::lowered_reaching(&reaching("", body), &common::held());

    assert!(
        common::body_of(&lowered, "go")
            .instructions
            .contains(&Instruction::New(ClassName::new(built))),
        "{body} builds {built}"
    );
}

#[test]
fn the_classes_of_a_type_another_module_declares_are_not_written_again() {
    let source = reaching("user: held.User", "user.name");

    let lowered = common::lowered_reaching(&source, &common::held());

    let written = common::written(&lowered);
    assert!(
        !written.iter().any(|class| class.starts_with("held/")),
        "the other module writes its own classes: {written:?}"
    );
}

#[test]
fn a_record_of_another_module_is_built_as_that_module_s_class() {
    builds("_ = held.User { name: \"world\" }", "held/User");
}

#[test]
fn a_field_of_a_record_of_another_module_is_read_off_that_module_s_class() {
    let source = reaching("user: held.User", "user.name");

    let lowered = common::lowered_reaching(&source, &common::held());

    let read = common::body_of(&lowered, "go")
        .instructions
        .iter()
        .find_map(|instruction| match instruction {
            Instruction::GetField(field) => Some(field),
            _ => None,
        })
        .expect("reading a field reads one");
    assert_eq!(read.class, ClassName::new("held/User"));
    assert_eq!(read.name, "name");
}

#[test]
fn a_variant_of_another_module_is_built_as_the_class_that_module_writes_it_as() {
    builds("_ = held.Sent(\"post\")", "held/Payment$Sent");
}

#[test]
fn a_variant_of_another_module_that_carries_nothing_is_built_by_its_name_alone() {
    builds("_ = held.Pending", "held/Payment$Pending");
}

#[test]
fn a_match_over_a_type_of_another_module_tests_the_tag_that_module_counts() {
    let source = concat!(
        "import held\n\n",
        "fn go(payment: held.Payment) -> String {\n",
        "    match payment {\n",
        "        held.Sent(how) => how\n",
        "        held.Pending => \"pending\"\n",
        "    }\n}\n"
    );

    let lowered = common::lowered_reaching(source, &common::held());

    let read = common::body_of(&lowered, "go")
        .instructions
        .iter()
        .find_map(|instruction| match instruction {
            Instruction::GetField(field) if field.name == "tag" => Some(field),
            _ => None,
        })
        .expect("a match over two variants reads the tag");
    assert_eq!(read.class, ClassName::new("held/Payment"));
    assert_eq!(read.of, Descriptor::Integer);
}
