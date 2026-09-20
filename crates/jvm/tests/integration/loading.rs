//! What a class asks a JVM to load before it settles its own layout.
//!
//! `docs/specs/codegen.md` states it: a JVM folds a value into the class holding it only where
//! it knows what that value holds, and `LoadableDescriptors` is how a class file asks for that.

use crate::common;

/// A record whose field is a record, which is the layout the attribute is written for.
const HOLDING_A_RECORD: &str = "fn number(user: User) -> Int {
    user.home.number
}

type User = {
    id: Int
    home: Address
}

type Address = {
    number: Int
    zone: Int
}
";

#[test]
fn a_class_holding_a_record_asks_for_that_record_to_be_loaded_first() {
    let files = common::compiled(HOLDING_A_RECORD);

    let user = common::one_of(&files, "demo/User.class");

    assert_eq!(user.loadable, vec!["Ldemo/Address;".to_owned()]);
}

#[test]
fn a_class_holding_nothing_this_build_writes_asks_for_nothing_at_all() {
    let files = common::compiled(HOLDING_A_RECORD);

    let address = common::one_of(&files, "demo/Address.class");

    assert!(
        address.loadable.is_empty(),
        "a record of whole numbers waits on nothing"
    );
}

#[test]
fn a_field_carried_by_a_class_the_jvm_ships_is_not_asked_for() {
    let source = "fn named(user: User) -> String {
    user.name
}

type User = {
    name: String
    count: Int
}
";

    let files = common::compiled(source);

    assert!(
        common::one_of(&files, "demo/User.class")
            .loadable
            .is_empty(),
        "`java.lang.String` is no value class, so nothing of it folds into `User`"
    );
}

#[test]
fn two_fields_of_one_type_ask_for_it_once() {
    let source = "fn across(line: Line) -> Int {
    line.end.across - line.start.across
}

type Line = {
    start: Point
    end: Point
}

type Point = {
    across: Int
    down: Int
}
";

    let files = common::compiled(source);

    let line = common::one_of(&files, "demo/Line.class");

    assert_eq!(line.loadable, vec!["Ldemo/Point;".to_owned()]);
}

#[test]
fn a_class_asks_for_what_it_holds_in_the_order_it_declares_its_fields() {
    let source = "fn across(place: Place) -> Int {
    place.at.across + place.size.across
}

type Place = {
    size: Size
    name: String
    at: Point
}

type Size = {
    across: Int
}

type Point = {
    across: Int
}
";

    let files = common::compiled(source);

    let place = common::one_of(&files, "demo/Place.class");

    assert_eq!(
        place.loadable,
        vec!["Ldemo/Size;".to_owned(), "Ldemo/Point;".to_owned()]
    );
}

#[test]
fn a_variant_holding_a_record_asks_for_it_the_way_a_record_does() {
    let source = "fn across(shape: Shape) -> Int {
    match shape {
        Round(at) => at.across
        Flat => 0
    }
}

type Shape =
    | Round(Point)
    | Flat

type Point = {
    across: Int
    down: Int
}
";

    let files = common::compiled(source);

    let round = common::one_of(&files, "demo/Shape$Round.class");

    assert_eq!(round.loadable, vec!["Ldemo/Point;".to_owned()]);
}

#[test]
fn the_module_class_asks_for_nothing_because_it_declares_no_field() {
    let files = common::compiled(HOLDING_A_RECORD);

    let module = common::one_of(&files, "demo.class");

    assert!(
        module.loadable.is_empty(),
        "a module class holds no value, so no layout of it waits on one"
    );
}

#[test]
fn a_record_holding_itself_does_not_ask_for_itself() {
    let source = "fn number(node: Node) -> Int {
    node.number
}

type Node = {
    number: Int
    next: Node
}
";

    let files = common::compiled(source);

    assert!(
        common::one_of(&files, "demo/Node.class")
            .loadable
            .is_empty(),
        "a class is the one being laid out, so nothing of it waits on it"
    );
}

#[test]
fn a_field_typed_as_the_base_of_a_sum_type_is_not_asked_for() {
    let source = "fn across(box: Box) -> Int {
    match box.held {
        Round(size) => size
        Flat => 0
    }
}

type Box = {
    held: Shape
}

type Shape =
    | Round(Int)
    | Flat
";

    let files = common::compiled(source);

    assert!(
        common::one_of(&files, "demo/Box.class").loadable.is_empty(),
        "a field holding whichever variant it was given stays a reference"
    );
}
