//! What each construct of version 0.1 becomes, which `docs/specs/codegen.md` states.

use lumen_ir::{ClassName, Descriptor, Extending, Instruction, Reached, THE_ONE_SHAPE};

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
    let source = "fn is_held(count: Int, name: String) -> Bool {\n    count > 1\n}\n\n\
                  fn is_negated(flag: Bool) -> Bool {\n    !flag\n}\n";

    let lowered = common::lowered(source);

    let demo = common::class_of(&lowered, &ClassName::new("demo"));
    assert_eq!(
        common::method_of(demo, "is_held").descriptor.to_string(),
        "(JLjava/lang/String;)Z"
    );
    assert_eq!(
        common::method_of(demo, "is_negated").descriptor.to_string(),
        "(Z)Z"
    );
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
fn a_field_a_type_parameter_leaves_open_is_object_so_a_number_put_in_one_is_boxed() {
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

/// A function whose result is a type parameter, which every use of it settles for itself.
///
/// It is written below the function that uses it, which is where a definition belongs.
const GENERIC: &str = "\nfn identity<T>(value: T) -> T {\n    value\n}\n";

#[test]
fn a_condition_that_came_through_a_generic_is_already_a_truth_value() {
    let source = format!(
        "fn picked(count: Int) -> Int {{\n    if identity(count > 1) {{\n        1\n    }} else {{\n        2\n    }}\n}}\n{GENERIC}"
    );

    let lowered = common::lowered(&source);

    let picked = common::body_of(&lowered, "picked");
    let boolean = ClassName::new("java/lang/Boolean");
    assert!(
        !common::calls(picked, &boolean, "booleanValue"),
        "the generic was written taking and giving back a word, so nothing is read back"
    );
}

#[test]
fn the_operand_of_not_came_back_from_the_generic_as_a_word() {
    let source =
        format!("fn is_negated(flag: Bool) -> Bool {{\n    !identity(flag)\n}}\n{GENERIC}");

    let lowered = common::lowered(&source);

    let negated = common::body_of(&lowered, "is_negated");
    let boolean = ClassName::new("java/lang/Boolean");
    assert!(!common::calls(negated, &boolean, "booleanValue"));
}

#[test]
fn an_operand_that_came_through_a_generic_is_already_a_whole_number() {
    let source =
        format!("fn is_compared(count: Int) -> Bool {{\n    identity(count) < 2\n}}\n{GENERIC}");

    let lowered = common::lowered(&source);

    let compared = common::body_of(&lowered, "is_compared");
    let long = ClassName::new("java/lang/Long");
    assert!(
        !common::calls(compared, &long, "longValue"),
        "lcmp takes what the generic gave back, which is already a whole number"
    );
}

#[test]
fn a_generic_used_at_one_type_is_written_once_for_that_type() {
    let source = format!("fn kept(count: Int) -> Int {{\n    identity(count)\n}}\n{GENERIC}");

    let lowered = common::lowered(&source);

    assert!(common::has_method(&lowered, "identity$Int"));
    assert!(!common::has_method(&lowered, "identity"));
}

#[test]
fn a_generic_used_at_two_types_is_written_once_for_each() {
    let source = format!(
        "fn kept(count: Int) -> Int {{\n    if identity(count > 0) {{\n        identity(count)\n    }} else {{\n        0\n    }}\n}}\n{GENERIC}"
    );

    let lowered = common::lowered(&source);

    assert!(common::has_method(&lowered, "identity$Int"));
    assert!(common::has_method(&lowered, "identity$Bool"));
}

#[test]
fn a_generic_used_twice_at_one_type_is_written_once() {
    let source = format!(
        "fn kept(count: Int) -> Int {{\n    identity(count) + identity(count)\n}}\n{GENERIC}"
    );

    let lowered = common::lowered(&source);

    assert_eq!(common::methods_named(&lowered, "identity$Int"), 1);
}

#[test]
fn a_generic_nothing_uses_is_written_not_at_all() {
    let source = format!("fn kept(count: Int) -> Int {{\n    count\n}}\n{GENERIC}");

    let lowered = common::lowered(&source);

    assert_eq!(common::methods_named(&lowered, "identity$Int"), 0);
    assert!(!common::has_method(&lowered, "identity"));
}

#[test]
fn a_generic_a_generic_calls_is_written_for_the_types_the_outer_one_was() {
    let source = concat!(
        "fn kept(count: Int) -> Int {\n    twice(count)\n}\n\n",
        "fn twice<T>(value: T) -> T {\n    identity(identity(value))\n}\n\n",
        "fn identity<T>(value: T) -> T {\n    value\n}\n"
    );

    let lowered = common::lowered(source);

    assert!(common::has_method(&lowered, "twice$Int"));
    assert!(common::has_method(&lowered, "identity$Int"));
}

#[test]
fn a_generic_naming_two_type_parameters_is_named_for_both_in_the_order_declared() {
    let source = concat!(
        "fn kept(count: Int) -> Int {\n    paired(count, \"two\")\n}\n\n",
        "fn paired<A, B>(first: A, second: B) -> A {\n    first\n}\n"
    );

    let lowered = common::lowered(source);

    assert!(common::has_method(&lowered, "paired$Int$String"));
}

#[test]
fn a_generic_used_at_nothing_interesting_takes_and_gives_back_nothing() {
    let source = format!("fn kept() -> () {{\n    identity(())\n}}\n{GENERIC}");

    let lowered = common::lowered(&source);

    let written = common::method_of(
        common::class_of(&lowered, &ClassName::new("demo")),
        "identity$Unit",
    );
    assert_eq!(written.descriptor.to_string(), "()V");
}

#[test]
fn a_generic_that_calls_itself_is_written_once_and_the_writing_ends() {
    let source = concat!(
        "fn kept(count: Int) -> Int {\n    counted(count, 2)\n}\n\n",
        "fn counted<T>(value: T, at: Int) -> Int {\n    if at > 0 {\n        counted(value, at + -1) + 1\n    } else {\n        0\n    }\n}\n"
    );

    let lowered = common::lowered(source);

    assert_eq!(common::methods_named(&lowered, "counted$Int"), 1);
}

#[test]
fn a_declared_type_named_for_nothing_interesting_does_not_take_that_name_from_it() {
    let source = concat!(
        "fn kept(tag: Unit) -> Unit {\n    identity(tag)\n}\n\n",
        "fn nothing() -> () {\n    identity(())\n}\n\n",
        "fn identity<T>(value: T) -> T {\n    value\n}\n\n",
        "type Unit = {\n    at: Int\n}\n"
    );

    let lowered = common::lowered(source);

    let written: Vec<String> = common::class_of(&lowered, &ClassName::new("demo"))
        .methods
        .iter()
        .filter(|method| method.name == "identity$Unit")
        .map(|method| method.descriptor.to_string())
        .collect();
    assert_eq!(
        written,
        ["(Ldemo/Unit;)Ldemo/Unit;", "()V"],
        "one name, two methods, told apart by what each takes"
    );
}

#[test]
fn a_main_that_declares_a_type_parameter_is_still_what_the_module_is_run_through() {
    let lowered = common::lowered("fn main<T>(arguments: List<String>) -> Int {\n    0\n}\n");

    assert!(common::has_method(&lowered, "main"));
    assert!(lumen_ir::is_a_program(&lowered));
}

#[test]
fn a_generic_used_at_one_type_with_two_arguments_is_written_once_for_that_type() {
    let source = concat!(
        "fn kept(count: Int) -> Int {\n    or(identity(Some(count)), 0) + flagged(identity(Some(count > 0)))\n}\n\n",
        "fn flagged(flag: Option<Bool>) -> Int {\n    if or(flag, false) {\n        1\n    } else {\n        0\n    }\n}\n\n",
        "fn identity<T>(value: T) -> T {\n    value\n}\n"
    );

    let lowered = common::lowered(source);

    assert_eq!(common::methods_named(&lowered, "identity$Option"), 1);
}

/// A module that can be run, which is one declaring the `main` a program starts at.
const PROGRAM: &str = "fn main(arguments: List<String>) -> Int {\n    0\n}\n";

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
        "(Ljava/util/List;)J",
        "the one written first"
    );
    assert!(lumen_ir::is_a_program(&lowered));
}

/// The text a refusal names and the type the lowering reads are one shape, and this is both.
#[test]
fn a_module_written_at_the_shape_a_refusal_names_is_the_program_that_refusal_asks_for() {
    let lowered = common::lowered(&format!("{THE_ONE_SHAPE} {{\n    0\n}}\n"));

    assert!(
        lumen_ir::is_a_program(&lowered),
        "`{THE_ONE_SHAPE}` is what a refusal tells an author to write"
    );
}

#[test]
fn a_module_declaring_main_at_another_shape_is_a_library_the_way_one_declaring_none_is() {
    let lowered = common::lowered("fn main() -> () {\n    ()\n}\n");

    assert!(
        !lumen_ir::is_a_program(&lowered),
        "a program starts at `fn main(arguments: List<String>) -> Int` and at no other shape"
    );
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

    let started = entry_point(module);
    assert_eq!(started.name, "main");
    assert!(common::calls(
        &started.body,
        &ClassName::new("demo"),
        "main"
    ));
}

#[test]
fn the_entry_point_gathers_what_a_jvm_hands_it_into_the_list_main_takes() {
    let lowered = common::lowered(PROGRAM);

    let started = entry_point(common::class_of(&lowered, &ClassName::new("demo")));

    assert_eq!(
        started.body.instructions[..2],
        [
            Instruction::Load {
                slot: 0,
                of: Descriptor::array(Descriptor::reference("java/lang/String")),
            },
            Instruction::CollectList,
        ],
        "the array a JVM hands it, gathered into a list"
    );
}

#[test]
fn the_entry_point_ends_the_run_with_the_low_eight_bits_of_what_main_gave_back() {
    let lowered = common::lowered(PROGRAM);

    let started = entry_point(common::class_of(&lowered, &ClassName::new("demo")));

    let ending = &started.body.instructions[started.body.instructions.len() - 3..];
    assert_eq!(ending[0], Instruction::LowEightBits, "a status is a byte");
    assert!(
        common::calls(&started.body, &ClassName::new("java/lang/System"), "exit"),
        "the run ends with the status the program gave back"
    );
    assert_eq!(ending[2], Instruction::Return(None));
}

/// The method a JVM starts the module class at, which is the one taking the array it hands over.
fn entry_point(module: &lumen_ir::Class) -> &lumen_ir::Method {
    module
        .methods
        .iter()
        .find(|method| method.descriptor.to_string() == "([Ljava/lang/String;)V")
        .expect("the module is written with an entry point")
}
