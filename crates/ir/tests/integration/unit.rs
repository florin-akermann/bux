//! What a `()` becomes where something holding a reference is handed one.

use lumen_ir::{ClassName, Instruction, MethodDescriptor, MethodRef};

use crate::common;

#[test]
fn a_unit_handed_to_something_that_holds_a_reference_stands_as_a_value_it_can_hold() {
    for (named, source) in common::HOLDING_NOTHING {
        let lowered = common::lowered(source);

        let body = common::body_of(&lowered, named);

        assert!(
            stands_for_nothing(&body.instructions),
            "`{named}` leaves nothing where a reference is wanted: {:?}",
            body.instructions
        );
    }
}

#[test]
fn a_unit_written_in_a_list_fills_the_element_it_is_written_at() {
    let lowered = common::lowered("fn written() -> List<()> {\n    [()]\n}\n");

    let body = common::body_of(&lowered, "written");

    assert_eq!(
        body.instructions,
        [
            Instruction::Integer(1),
            Instruction::NewArray(standing_for_nothing()),
            Instruction::Copy,
            Instruction::Integer(0),
            Instruction::New(standing_for_nothing()),
            Instruction::Copy,
            Instruction::Construct(built()),
            Instruction::StoreInArray,
            common::building_a_list(),
            Instruction::Return(Some(common::a_list())),
        ]
    );
}

#[test]
fn a_unit_given_to_something_that_holds_none_stands_as_nothing_at_all() {
    let lowered = common::lowered(
        "fn is_asked() -> Bool {\n    is_given(())\n}\n\n// example: is_given(())\nfn is_given(given: ()) -> Bool {\n    true\n}\n",
    );

    let body = common::body_of(&lowered, "is_asked");

    assert!(
        !stands_for_nothing(&body.instructions),
        "nothing stands for a `()` a parameter does not carry: {:?}",
        body.instructions
    );
}

/// Whether the instructions build the value a `()` stands as, in the order they must.
fn stands_for_nothing(instructions: &[Instruction]) -> bool {
    let run = [
        Instruction::New(standing_for_nothing()),
        Instruction::Copy,
        Instruction::Construct(built()),
    ];
    instructions.windows(run.len()).any(|held| held == run)
}

/// The class a `()` stands as where a reference is wanted, which nothing reads back out.
fn standing_for_nothing() -> ClassName {
    ClassName::new("java/lang/Object")
}

/// The constructor that builds it, which takes nothing because a `()` holds nothing.
fn built() -> MethodRef {
    MethodRef {
        class: standing_for_nothing(),
        name: "<init>".to_owned(),
        descriptor: MethodDescriptor::new(Vec::new(), None),
    }
}
