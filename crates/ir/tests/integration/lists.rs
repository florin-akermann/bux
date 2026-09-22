//! What a written list becomes, and what growing one and reading one at an index become.
//!
//! `docs/specs/codegen.md` states all three. `list.push` and `list.at` are the compiler's, which
//! `docs/specs/library.md` states, so a call of either is written out where it stands rather than
//! reached on the `list` class.

use lumen_ir::{ClassName, Instruction, Lowered};

use crate::common;

#[test]
fn a_written_list_gathers_its_elements_into_an_array_and_collects_them() {
    let lowered = common::lowered("fn words() -> List<String> {\n    [\"a\", \"b\"]\n}\n");

    let body = common::body_of(&lowered, "words");

    assert_eq!(
        body.instructions
            .iter()
            .filter(|step| **step == Instruction::StoreInArray)
            .count(),
        2,
        "one store per element"
    );
    assert_eq!(
        body.instructions.first(),
        Some(&Instruction::Integer(2)),
        "the array is as long as the list is"
    );
    assert!(
        body.instructions
            .contains(&Instruction::NewArray(ClassName::new("java/lang/Object")))
    );
    assert!(body.instructions.contains(&common::building_a_list()));
}

#[test]
fn a_written_list_of_nothing_gathers_an_array_of_nothing() {
    let lowered = common::lowered("fn none() -> List<Int> {\n    []\n}\n");

    let body = common::body_of(&lowered, "none");

    assert_eq!(
        body.instructions,
        [
            Instruction::Integer(0),
            Instruction::NewArray(ClassName::new("java/lang/Object")),
            common::building_a_list(),
            Instruction::Return(Some(common::a_list())),
        ]
    );
}

#[test]
fn a_whole_number_written_in_a_list_is_boxed_on_the_way_in() {
    let lowered = common::lowered("fn counts() -> List<Int> {\n    [1]\n}\n");

    let body = common::body_of(&lowered, "counts");

    assert!(common::calls(
        body,
        &ClassName::new("java/lang/Long"),
        "valueOf"
    ));
}

#[test]
fn a_written_list_fills_its_array_left_to_right() {
    let lowered = common::lowered("fn words() -> List<String> {\n    [\"a\", \"b\"]\n}\n");

    let body = common::body_of(&lowered, "words");
    let filled: Vec<&Instruction> = body
        .instructions
        .iter()
        .filter(|step| matches!(step, Instruction::Text(_) | Instruction::Integer(_)))
        .collect();

    assert_eq!(
        filled,
        [
            &Instruction::Integer(2),
            &Instruction::Integer(0),
            &Instruction::Text("a".to_owned()),
            &Instruction::Integer(1),
            &Instruction::Text("b".to_owned()),
        ]
    );
}

#[test]
fn a_push_asks_the_list_module_for_no_method() {
    let lowered = pushing();

    let body = common::body_of(&lowered, "read");

    assert!(!reaches_the_list_module(body));
}

#[test]
fn a_push_is_one_call_of_the_class_that_carries_a_list() {
    let lowered = pushing();

    let body = common::body_of(&lowered, "read");

    let reached: Vec<String> = body
        .instructions
        .iter()
        .filter_map(common::called)
        .filter(|method| method.class != ClassName::new("java/lang/Long"))
        .map(|method| format!("{}.{}{}", method.class, method.name, method.descriptor))
        .collect();
    assert_eq!(
        reached,
        ["lumen/List.push(Llumen/List;Ljava/lang/Object;)Llumen/List;"]
    );
}

#[test]
fn a_read_at_an_index_asks_the_list_module_for_no_method() {
    let lowered = reading();

    let body = common::body_of(&lowered, "read");

    assert!(!reaches_the_list_module(body));
}

#[test]
fn a_read_at_an_index_reads_the_length_and_one_slot_of_the_buffer() {
    let lowered = reading();

    let body = common::body_of(&lowered, "read");

    let read: Vec<&str> = body
        .instructions
        .iter()
        .filter_map(|step| match step {
            Instruction::GetField(field) if field.class == ClassName::new("lumen/List") => {
                Some(field.name.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(read, ["length", "slots"]);
    assert_eq!(
        body.instructions
            .iter()
            .filter(|step| **step == Instruction::LoadFromArray)
            .count(),
        1,
        "one slot, whatever the list holds"
    );
}

#[test]
fn every_class_carries_a_list_in_a_buffer_and_a_length() {
    let lowered = pushing();

    let carrier = common::class_of(&lowered, &ClassName::new("lumen/List"));

    assert_eq!(common::holds(carrier), ["slots", "length"]);
    let methods: Vec<&str> = carrier
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect();
    assert_eq!(methods, ["<init>", "of", "push", "listed"]);
}

#[test]
fn a_read_at_an_index_narrows_it_to_the_width_the_member_takes() {
    let lowered = reading();

    let body = common::body_of(&lowered, "read");

    assert!(body.instructions.contains(&Instruction::Narrow));
}

#[test]
fn a_read_at_an_index_builds_both_of_the_answers_an_option_carries() {
    let lowered = reading();

    let body = common::body_of(&lowered, "read");
    let built: Vec<&ClassName> = body
        .instructions
        .iter()
        .filter_map(|step| match step {
            Instruction::New(class) => Some(class),
            _ => None,
        })
        .collect();

    assert_eq!(
        built,
        [
            &ClassName::new("lumen/Option$Some"),
            &ClassName::new("lumen/Option$None"),
        ]
    );
}

/// A module importing `list` and giving back what `written` works out.
fn reaching_list(written: &str, gives: &str) -> String {
    format!("import list\n\nfn read(values: List<Int>) -> {gives} {{\n    {written}\n}}\n")
}

/// What that module becomes, with what `list` offers reachable through an import of it.
fn lowered_over_list(written: &str, gives: &str) -> Lowered {
    common::lowered_reaching(
        &reaching_list(written, gives),
        &common::library("list").offered(),
    )
}

/// A module whose one function grows a list, which is what the pushes above are read out of.
fn pushing() -> Lowered {
    lowered_over_list("list.push(values, 1)", "List<Int>")
}

/// A module whose one function reads a list at an index, read out the same way.
fn reading() -> Lowered {
    lowered_over_list("list.at(values, 1)", "Option<Int>")
}

/// Whether the body calls any method of the `list` class, which neither of the two is.
fn reaches_the_list_module(body: &lumen_ir::Body) -> bool {
    body.instructions
        .iter()
        .filter_map(common::called)
        .any(|reference| reference.class == ClassName::new("list"))
}
