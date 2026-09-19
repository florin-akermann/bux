//! What the JVM is told a value is, which `docs/specs/codegen.md` states.

use lumen_ir::{ClassName, Descriptor, MethodDescriptor};

#[test]
fn a_whole_number_is_told_to_the_jvm_as_a_long() {
    assert_eq!(Descriptor::Long.to_string(), "J");
    assert_eq!(Descriptor::Long.width(), 2, "a long takes two slots");
}

#[test]
fn a_truth_value_is_told_to_the_jvm_as_a_boolean() {
    assert_eq!(Descriptor::Boolean.to_string(), "Z");
    assert_eq!(Descriptor::Boolean.width(), 1);
}

#[test]
fn a_reference_names_the_class_it_points_at() {
    let string = Descriptor::reference("java/lang/String");

    assert_eq!(string.to_string(), "Ljava/lang/String;");
}

#[test]
fn a_method_descriptor_runs_its_parameters_together_and_ends_with_its_result() {
    let descriptor = MethodDescriptor::new(
        vec![Descriptor::Long, Descriptor::Boolean],
        Some(Descriptor::reference("java/lang/String")),
    );

    assert_eq!(descriptor.to_string(), "(JZ)Ljava/lang/String;");
    assert_eq!(descriptor.width(), 3, "a long, then a boolean");
}

#[test]
fn a_method_giving_back_nothing_ends_with_void() {
    let descriptor = MethodDescriptor::new(vec![Descriptor::Long], None);

    assert_eq!(descriptor.to_string(), "(J)V");
}

#[test]
fn a_class_is_written_to_the_path_its_name_gives() {
    assert_eq!(ClassName::new("demo/User").path(), "demo/User.class");
}
