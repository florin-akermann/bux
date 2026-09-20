//! A declared type that holds a value of itself, which `docs/specs/types.md` refuses.

use crate::common::{inferred, refusal};

/// A module declaring `held` and reading one field of the first type it declares.
fn declaring(held: &str) -> String {
    format!("fn counted(node: Node) -> Int {{\n    node.count\n}}\n\n{held}")
}

/// The one record of the module, whose second field is written `held`.
fn holding(held: &str) -> String {
    declaring(&format!(
        "type Node = {{\n    count: Int\n    next: {held}\n}}\n"
    ))
}

#[test]
fn a_record_holding_one_of_itself_is_refused_where_it_is_declared() {
    let error = refusal(&holding("Node"));

    assert_eq!(error.message(), "`Node` holds `Node`");
}

#[test]
fn the_refusal_points_at_the_name_the_type_is_declared_under() {
    let source = holding("Node");

    let at = refusal(&source).span();

    assert_eq!(at.text(&source), "Node");
}

#[test]
fn a_ring_of_records_is_refused_and_named_in_the_order_it_runs() {
    let source = declaring(
        "type Node = {
    count: Int
    next: Wing
}

type Wing = {
    back: Node
}
",
    );

    assert_eq!(
        refusal(&source).message(),
        "`Node` holds `Wing`, which holds `Node`"
    );
}

#[test]
fn a_longer_ring_names_every_type_it_runs_through() {
    let source = declaring(
        "type Node = {
    count: Int
    next: Wing
}

type Wing = {
    side: Room
}

type Room = {
    back: Node
}
",
    );

    assert_eq!(
        refusal(&source).message(),
        "`Node` holds `Wing`, which holds `Room`, which holds `Node`"
    );
}

#[test]
fn a_record_holding_an_option_of_itself_is_how_one_holds_its_own_kind() {
    inferred(&holding("Option<Node>"));
}

#[test]
fn a_chain_of_records_that_never_comes_back_round_is_accepted() {
    let source = declaring(
        "type Node = {
    count: Int
    next: Wing
}

type Wing = {
    side: Room
}

type Room = {
    count: Int
}
",
    );

    inferred(&source);
}

#[test]
fn a_field_written_as_the_base_of_an_adt_holds_a_reference_and_is_no_ring() {
    let source = declaring(
        "type Node = {
    count: Int
    next: Shape
}

type Shape =
    | Round(Node)
    | Flat
",
    );

    inferred(&source);
}

#[test]
fn a_record_held_as_a_type_argument_is_not_held_by_value() {
    let source = declaring(
        "type Node = {
    count: Int
    next: List<Node>
}
",
    );

    inferred(&source);
}

#[test]
fn a_ring_is_found_however_far_down_the_file_it_is_declared() {
    let source = declaring(
        "type Node = {
    count: Int
}

type Wing = {
    side: Room
}

type Room = {
    back: Wing
}
",
    );

    assert_eq!(
        refusal(&source).message(),
        "`Wing` holds `Room`, which holds `Wing`"
    );
}

#[test]
fn a_record_that_leads_into_a_ring_is_not_itself_what_the_ring_is_reported_against() {
    let source = declaring(
        "type Node = {
    count: Int
    next: Wing
}

type Wing = {
    side: Room
}

type Room = {
    back: Wing
}
",
    );

    let error = refusal(&source);

    assert_eq!(error.message(), "`Wing` holds `Room`, which holds `Wing`");
    assert_eq!(error.span().text(&source), "Wing");
}

/// A module of `many` records, each holding the next one twice, and the last holding nothing.
///
/// Every route through it is a different way of picking one of the two fields at each step, so
/// there are two to the power of `many` of them, and a walk that follows routes rather than
/// records never comes back. Nothing here is a ring, and the module is an ordinary one.
fn widening(many: usize) -> String {
    let declared = (0..many).map(|at| {
        let held = if at + 1 == many {
            "Int".to_owned()
        } else {
            format!("Held{}", at + 1)
        };
        format!("\ntype Held{at} = {{\n    count: Int\n    one: {held}\n    two: {held}\n}}\n")
    });
    declared.fold(
        "fn counted(held: Held0) -> Int {\n    held.count\n}\n".to_owned(),
        |mut source, written| {
            source.push_str(&written);
            source
        },
    )
}

#[test]
fn a_record_many_others_hold_is_gone_into_once_rather_than_once_per_route_to_it() {
    inferred(&widening(24));
}
