//! What a written list becomes, which `docs/specs/codegen.md` states.

use lumen_ir::{ClassName, Descriptor, Instruction};

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
    assert!(body.instructions.contains(&Instruction::CollectList));
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
            Instruction::CollectList,
            Instruction::Return(Some(Descriptor::reference("java/util/List"))),
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
