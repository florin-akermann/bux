//! What a whole module looks like once it is written, read back out of the bytes.

use crate::common;
use crate::reader;
use crate::reader::ClassFile;

/// `ACC_PUBLIC` and `ACC_FINAL`, and no identity bit: a value class nothing extends.
const FINAL_CLASS: u16 = 0x0011;

/// `ACC_PUBLIC` and `ACC_ABSTRACT`, and no identity bit: the value class a type's variants extend.
const BASE_CLASS: u16 = 0x0401;

const PUBLIC_STATIC: u16 = 0x0009;

/// `ACC_PUBLIC`, `ACC_FINAL`, and `ACC_STRICT_INIT`, which every field of a value class has.
const PUBLIC_FINAL_STRICT: u16 = 0x0811;

/// `ACC_SYNCHRONIZED`, which locks on an object, and a value has no identity to be locked on.
const SYNCHRONIZED: u16 = 0x0020;

/// A module of one function over a record and one over an algebraic data type.
const MODULE: &str = "fn counted(users: List<User>) -> Int {\n    var total = 0\n    for user in users {\n        total += 1\n    }\n    total\n}\n\nfn told(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n\ntype User = {\n    id: Int\n    active: Bool\n}\n";

#[test]
fn every_function_of_a_module_is_a_public_static_method_of_the_module_class() {
    let files = common::compiled(MODULE);

    let module = common::one_of(&files, "demo.class");

    let named: Vec<&str> = module
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect();
    assert_eq!(named, ["counted", "told"], "in the order they are written");
    assert_eq!(module.methods[0].access, PUBLIC_STATIC);
    assert_eq!(module.methods[0].descriptor, "(Ljava/util/List;)J");
    assert_eq!(
        module.methods[1].descriptor,
        "(Ldemo/Payment;)Ljava/lang/String;"
    );
}

#[test]
fn a_record_is_a_final_class_whose_fields_are_final() {
    let files = common::compiled(MODULE);

    let user = common::one_of(&files, "demo/User.class");

    assert_eq!(user.access, FINAL_CLASS);
    assert_eq!(user.class(user.extends), "java/lang/Object");
    assert_eq!(user.field("id").access, PUBLIC_FINAL_STRICT);
    assert_eq!(user.field("id").descriptor, "J");
    assert_eq!(user.field("active").descriptor, "Z");
}

#[test]
fn a_variant_is_a_final_class_extending_the_base_of_its_type() {
    let files = common::compiled(MODULE);

    let base = common::one_of(&files, "demo/Payment.class");
    let failed = common::one_of(&files, "demo/Payment$Failed.class");

    assert_eq!(base.access, BASE_CLASS, "only its own variants extend it");
    assert_eq!(base.field("tag").descriptor, "I");
    assert_eq!(failed.access, FINAL_CLASS);
    assert_eq!(failed.class(failed.extends), "demo/Payment");
    assert_eq!(failed.field("value0").descriptor, "Ljava/lang/String;");
}

#[test]
fn a_method_that_branches_carries_the_stack_map_the_verifier_wants() {
    let files = common::compiled(MODULE);

    let module = common::one_of(&files, "demo.class");

    let told = &module.method("told").code;
    assert!(
        told.as_ref()
            .expect("this method has a body")
            .frames
            .is_some(),
        "a match branches, so the places it lands are written down"
    );
}

#[test]
fn the_prelude_is_written_with_the_module_so_that_a_build_is_self_contained() {
    let files = common::compiled(MODULE);

    let written: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();

    assert!(written.contains(&"lumen/Option.class"), "{written:?}");
    assert!(written.contains(&"lumen/Option$Some.class"));
    assert!(written.contains(&"lumen/Result$Err.class"));
}

/// The writer fixes its method access today, so this holds the rule rather than exercising it.
///
/// It is the guard for the day something computes a method's flags: a value class cannot be
/// locked on, so `ACC_SYNCHRONIZED` is a bit no class Lumen writes may ever carry.
#[test]
fn no_method_a_module_writes_is_synchronized_because_a_value_cannot_be_locked_on() {
    for file in common::compiled(MODULE) {
        let class = reader::read(&file.bytes);
        for method in &class.methods {
            assert_eq!(
                method.access & SYNCHRONIZED,
                0,
                "{} declares `{}` synchronized",
                file.path,
                method.name
            );
        }
    }
}

/// A module using one generic at a whole number, a truth value, and a record it declares.
const GENERIC: &str = "fn kept(count: Int) -> Int {\n    identity(count)\n}\n\nfn is_kept(flag: Bool) -> Bool {\n    identity(flag)\n}\n\nfn tagged(tag: Tag) -> Tag {\n    identity(tag)\n}\n\nfn identity<T>(value: T) -> T {\n    value\n}\n\ntype Tag = {\n    at: Int\n}\n";

#[test]
fn a_generic_is_written_once_per_set_of_types_it_is_used_at() {
    let files = common::compiled(GENERIC);

    let written = named(&common::one_of(&files, "demo.class"));

    assert_eq!(
        written,
        [
            "kept",
            "is_kept",
            "tagged",
            "identity$Int",
            "identity$Bool",
            "identity$Tag"
        ],
        "one method per set of types used at, and none for the declaration itself"
    );
}

#[test]
fn an_int_passed_to_a_generic_is_carried_as_a_long_throughout() {
    let files = common::compiled(GENERIC);
    let module = common::one_of(&files, "demo.class");

    let written: Vec<&str> = ["identity$Int", "identity$Bool", "identity$Tag"]
        .iter()
        .map(|name| descriptor_of(&module, name))
        .collect();

    assert_eq!(written, ["(J)J", "(Z)Z", "(Ldemo/Tag;)Ldemo/Tag;"]);
}

#[test]
fn no_method_written_for_a_function_the_module_declares_names_object() {
    let files = common::compiled(GENERIC);

    let module = common::one_of(&files, "demo.class");

    for method in &module.methods {
        assert!(
            !method.descriptor.contains("java/lang/Object"),
            "{} is written {}",
            method.name,
            method.descriptor
        );
    }
}

/// The names of the methods the class declares, in the order it declares them.
fn named(module: &ClassFile) -> Vec<String> {
    module
        .methods
        .iter()
        .map(|method| method.name.clone())
        .collect()
}

/// How the class declares the method called `name`, which it declares exactly once.
fn descriptor_of<'a>(module: &'a ClassFile, name: &str) -> &'a str {
    let written = module
        .methods
        .iter()
        .find(|method| method.name == name)
        .unwrap_or_else(|| panic!("the module writes a method named {name}"));
    written.descriptor.as_str()
}
