//! What each construct of version 0.1 becomes, which `docs/specs/codegen.md` states.

use lumen_ir::{ClassName, Descriptor, Extending, Instruction, Reached};

use crate::common;

#[test]
fn a_module_becomes_one_class_holding_a_method_for_each_function() {
    let lowered = common::lowered("fn answer() -> Int {\n    7\n}\n");

    let module = common::class_of(&lowered, &ClassName::new("demo"));

    assert_eq!(module.extends.written(), "java/lang/Object");
    assert_eq!(module.methods.len(), 1, "one function is one method");
    assert_eq!(module.methods[0].name, "answer");
}

#[test]
fn a_function_is_reached_through_the_module_class_and_never_an_instance() {
    let lowered = common::lowered("fn answer() -> Int {\n    7\n}\n");

    let answer = common::method_of(
        common::class_of(&lowered, &ClassName::new("demo")),
        "answer",
    );

    assert_eq!(answer.reached, Reached::ThroughTheClass);
}

#[test]
fn a_signature_says_what_each_lumen_type_is_carried_by() {
    let source = "fn held(count: Int, active: Bool, name: String) -> Bool {\n    active\n}\n";

    let lowered = common::lowered(source);

    let held = common::method_of(common::class_of(&lowered, &ClassName::new("demo")), "held");
    assert_eq!(held.descriptor.to_string(), "(JZLjava/lang/String;)Z");
}

#[test]
fn a_function_giving_back_nothing_interesting_is_a_method_giving_back_void() {
    let lowered = common::lowered("fn nothing() -> () {\n    ()\n}\n");

    let nothing = common::method_of(
        common::class_of(&lowered, &ClassName::new("demo")),
        "nothing",
    );

    assert_eq!(nothing.descriptor.to_string(), "()V");
}

/// A record of one field of each kind version 0.1 can write down.
const RECORD: &str = "type User = {\n    id: Int\n    active: Bool\n}\n";

#[test]
fn a_record_becomes_a_final_class_with_one_field_per_field_it_declares() {
    let lowered = common::lowered(RECORD);

    let user = common::class_of(&lowered, &ClassName::new("demo/User"));

    assert_eq!(user.extending, Extending::Never);
    assert_eq!(common::holds(user), ["id", "active"], "as declared");
}

#[test]
fn a_record_field_is_carried_by_whatever_carries_its_lumen_type() {
    let lowered = common::lowered(RECORD);

    let user = common::class_of(&lowered, &ClassName::new("demo/User"));

    assert_eq!(user.fields[0].of, Descriptor::Long);
    assert_eq!(user.fields[1].of, Descriptor::Boolean);
}

#[test]
fn a_record_is_built_by_a_constructor_taking_its_fields_in_order() {
    let lowered = common::lowered(RECORD);

    let built = common::method_of(
        common::class_of(&lowered, &ClassName::new("demo/User")),
        "<init>",
    );

    assert_eq!(built.descriptor.to_string(), "(JZ)V");
}

#[test]
fn a_constructor_fills_its_fields_before_it_hands_itself_up_to_its_base() {
    let lowered = common::lowered(RECORD);

    let built = common::method_of(
        common::class_of(&lowered, &ClassName::new("demo/User")),
        "<init>",
    );

    let instructions = &built.body.instructions;
    let last_field = instructions
        .iter()
        .rposition(|instruction| matches!(instruction, Instruction::PutField(_)))
        .expect("the fields are written");
    let handed_up = instructions
        .iter()
        .position(|instruction| matches!(instruction, Instruction::Construct(_)))
        .expect("the base is constructed");
    assert!(
        last_field < handed_up,
        "a value class is whole before its base runs: {instructions:?}"
    );
}

/// An algebraic data type with one variant that carries nothing and one that carries a value.
const PAYMENT: &str = "type Payment =\n    | Pending\n    | Failed(String)\n";

#[test]
fn an_algebraic_data_type_becomes_a_base_class_holding_a_tag() {
    let lowered = common::lowered(PAYMENT);

    let base = common::class_of(&lowered, &ClassName::new("demo/Payment"));

    assert_eq!(base.extending, Extending::ByItsVariants);
    assert_eq!(common::holds(base), ["tag"]);
    assert_eq!(base.fields[0].of, Descriptor::Integer);
}

#[test]
fn a_variant_becomes_a_final_class_extending_the_type_that_declares_it() {
    let lowered = common::lowered(PAYMENT);

    let failed = common::class_of(&lowered, &ClassName::new("demo/Payment$Failed"));

    assert_eq!(
        failed.extends,
        common::class_of(&lowered, &ClassName::new("demo/Payment")).name
    );
    assert_eq!(failed.extending, Extending::Never);
}

#[test]
fn a_variant_names_what_it_carries_after_where_it_carries_it() {
    let lowered = common::lowered(PAYMENT);

    let failed = common::class_of(&lowered, &ClassName::new("demo/Payment$Failed"));

    assert_eq!(
        common::holds(failed),
        ["value0"],
        "carried in order, unnamed"
    );
    assert_eq!(
        common::method_of(failed, "<init>").descriptor.to_string(),
        "(Ljava/lang/String;)V"
    );
}

#[test]
fn a_variant_is_named_after_its_type_so_that_the_two_may_share_a_name() {
    let lowered = common::lowered("type UserId = UserId(Int)\n");

    let written = common::written(&lowered);

    assert!(written.contains(&"demo/UserId"), "the type: {written:?}");
    assert!(written.contains(&"demo/UserId$UserId"), "the variant");
}

#[test]
fn the_prelude_types_are_written_with_every_module() {
    let lowered = common::lowered("fn answer() -> Int {\n    7\n}\n");

    let written = common::written(&lowered);

    assert_eq!(
        written,
        [
            "demo",
            "lumen/Option",
            "lumen/Option$Some",
            "lumen/Option$None",
            "lumen/Result",
            "lumen/Result$Ok",
            "lumen/Result$Err",
        ]
    );
}

#[test]
fn a_record_class_declares_its_constructor_and_nothing_else() {
    let lowered = common::lowered("type User = {\n    id: Int\n}\n");

    let user = common::class_of(&lowered, &ClassName::new("demo/User"));

    assert_eq!(
        names_of(user),
        ["<init>"],
        "`==` is `Eq`, which `User` has not"
    );
}

#[test]
fn no_class_a_module_writes_declares_a_method_beyond_its_constructor() {
    let lowered = common::lowered(concat!(
        "type User = {\n    id: Int\n}\n\n",
        "type Payment =\n    | Pending\n    | Failed(String)\n"
    ));

    for class in &lowered.classes {
        let declared = names_of(class);
        assert!(
            declared.iter().all(|name| *name == "<init>"),
            "{} declares {declared:?}",
            class.name.written()
        );
    }
}

/// The names of the methods `class` declares, in the order it declares them.
fn names_of(class: &lumen_ir::Class) -> Vec<&str> {
    class
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect()
}

#[test]
fn a_type_parameter_erases_to_object_so_a_call_boxes_the_number_it_passes() {
    let lowered = common::lowered("fn held() -> Option<Int> {\n    Some(1)\n}\n");

    let held = common::body_of(&lowered, "held");

    assert!(
        common::calls(held, &ClassName::new("java/lang/Long"), "valueOf"),
        "a whole number passed where a type parameter is wanted is boxed"
    );
}

#[test]
fn a_value_read_back_at_a_settled_type_is_unboxed() {
    let source = "fn out(held: Option<Int>) -> Int {\n    match held {\n        Some(value) => value\n        None => 0\n    }\n}\n";

    let lowered = common::lowered(source);

    let out = common::body_of(&lowered, "out");
    assert!(common::calls(
        out,
        &ClassName::new("java/lang/Long"),
        "longValue"
    ));
}

#[test]
fn a_match_that_runs_out_of_arms_throws_rather_than_running_on() {
    let source = "fn out(held: Option<Int>) -> Int {\n    match held {\n        Some(value) => value\n        None => 0\n    }\n}\n";

    let lowered = common::lowered(source);

    let out = common::body_of(&lowered, "out");
    assert!(out.instructions.contains(&Instruction::Throw));
    assert!(common::calls(
        out,
        &ClassName::new("java/lang/AssertionError"),
        "<init>"
    ));
}

#[test]
fn a_for_in_counts_through_the_list_it_was_given() {
    let source = "fn walked(users: List<Int>) -> () {\n    for user in users {\n        continue\n    }\n}\n";

    let lowered = common::lowered(source);

    let walked = common::body_of(&lowered, "walked");
    assert!(common::calls(
        walked,
        &ClassName::new("java/util/List"),
        "size"
    ));
    assert!(common::calls(
        walked,
        &ClassName::new("java/util/List"),
        "get"
    ));
    assert!(
        walked
            .instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Increment { .. })),
        "the count goes up once a turn"
    );
}

#[test]
fn a_record_update_builds_another_one_out_of_the_fields_it_is_not_given() {
    let source = "fn renamed(user: User) -> User {\n    user { active: false }\n}\n\ntype User = {\n    id: Int\n    active: Bool\n}\n";

    let lowered = common::lowered(source);

    let renamed = common::body_of(&lowered, "renamed");
    assert!(
        renamed
            .instructions
            .contains(&Instruction::New(ClassName::new("demo/User"))),
        "another one is built rather than the old one changed"
    );
    assert!(
        renamed.instructions.iter().any(|instruction| matches!(
            instruction,
            Instruction::GetField(field) if field.name == "id"
        )),
        "the field no value was given is read off the old one"
    );
}

#[test]
fn a_question_mark_gives_back_the_error_it_was_handed() {
    let source = "fn used() -> Result<Int, String> {\n    value := held()?\n    Ok(value)\n}\n\nfn held() -> Result<Int, String> {\n    Ok(1)\n}\n";

    let lowered = common::lowered(source);

    let used = common::body_of(&lowered, "used");
    let returns = used
        .instructions
        .iter()
        .filter(|instruction| matches!(instruction, Instruction::Return(_)))
        .count();
    assert!(returns >= 2, "one for the error, one for the value");
    assert!(
        used.instructions.iter().any(|instruction| matches!(
            instruction,
            Instruction::Cast(class) if class.written() == "lumen/Result$Ok"
        )),
        "the value is read out of the one that worked"
    );
}

#[test]
fn two_lowerings_of_one_source_are_the_same() {
    let source = "fn held(user: User) -> Int {\n    user.id\n}\n\ntype User = {\n    id: Int\n}\n";

    assert_eq!(common::lowered(source), common::lowered(source));
}

/// A function whose result is a type parameter, so every use of it leaves a reference behind.
///
/// It is written below the function that uses it, which is where a definition belongs.
const ERASED: &str = "\nfn identity<T>(value: T) -> T {\n    value\n}\n";

#[test]
fn a_condition_whose_type_was_erased_is_read_back_as_a_truth_value() {
    let source = format!(
        "fn picked(flag: Bool) -> Int {{\n    if identity(flag) {{\n        1\n    }} else {{\n        2\n    }}\n}}\n{ERASED}"
    );

    let lowered = common::lowered(&source);

    let picked = common::body_of(&lowered, "picked");
    let boolean = ClassName::new("java/lang/Boolean");
    assert!(
        common::calls(picked, &boolean, "booleanValue"),
        "a jump reads a word, not a reference"
    );
}

#[test]
fn the_operand_of_not_is_read_back_before_it_is_flipped() {
    let source = format!("fn negated(flag: Bool) -> Bool {{\n    !identity(flag)\n}}\n{ERASED}");

    let lowered = common::lowered(&source);

    let negated = common::body_of(&lowered, "negated");
    let boolean = ClassName::new("java/lang/Boolean");
    assert!(common::calls(negated, &boolean, "booleanValue"));
}

#[test]
fn an_operand_whose_type_was_erased_is_read_back_before_the_operator_works_on_it() {
    let source =
        format!("fn compared(count: Int) -> Bool {{\n    identity(count) < 2\n}}\n{ERASED}");

    let lowered = common::lowered(&source);

    let compared = common::body_of(&lowered, "compared");
    let long = ClassName::new("java/lang/Long");
    assert!(
        common::calls(compared, &long, "longValue"),
        "lcmp takes two whole numbers"
    );
}

/// A module that can be run, which is one declaring the `main` a program starts at.
const PROGRAM: &str = "fn main() -> () {\n    ()\n}\n";

#[test]
fn a_module_declaring_main_is_written_with_the_entry_point_a_jvm_starts_at() {
    let lowered = common::lowered(PROGRAM);

    let module = common::class_of(&lowered, &ClassName::new("demo"));

    let started = common::method_of(module, "main");
    assert_eq!(
        module
            .methods
            .iter()
            .filter(|method| method.name == "main")
            .count(),
        2,
        "the function the module declares, and the entry point that calls it"
    );
    assert_eq!(
        started.descriptor.to_string(),
        "()V",
        "the one written first"
    );
    assert!(lumen_ir::is_a_program(&lowered));
}

#[test]
fn a_module_declaring_no_main_is_written_without_an_entry_point() {
    let lowered = common::lowered("fn answer() -> Int {\n    7\n}\n");

    let module = common::class_of(&lowered, &ClassName::new("demo"));

    assert_eq!(module.methods.len(), 1, "only the function it declares");
    assert!(!lumen_ir::is_a_program(&lowered));
}

#[test]
fn the_entry_point_takes_what_a_jvm_hands_a_program_and_calls_the_main_beside_it() {
    let lowered = common::lowered(PROGRAM);

    let module = common::class_of(&lowered, &ClassName::new("demo"));

    let started = module
        .methods
        .iter()
        .find(|method| method.descriptor.to_string() == "([Ljava/lang/String;)V")
        .expect("the module is written with an entry point");
    assert_eq!(started.name, "main");
    assert!(common::calls(
        &started.body,
        &ClassName::new("demo"),
        "main"
    ));
}
