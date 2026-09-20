//! What a derive writes, which `docs/specs/derive.md` makes the instance an author would have.

use std::fmt::Write as _;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{Body, ClassName, Comparison, Instruction};

use crate::common;

/// A comparison of two of a record with one `Int` field, over a module that derives its `Eq`.
const POINT: &str = concat!(
    "fn is_corner(point: Point) -> Bool {\n    point == point\n}\n\n",
    "derive Eq for Point\n\n",
    "type Point = {\n    across: Int\n    up: Int\n}\n"
);

#[test]
fn a_derive_writes_the_method_an_instance_of_the_same_trait_and_type_would() {
    let lowered = common::lowered(POINT);

    assert_eq!(common::methods_named(&lowered, "Eq$Point$is_equal"), 1);
}

#[test]
fn a_comparison_of_a_derived_type_calls_the_method_the_derive_wrote() {
    let lowered = common::lowered(POINT);

    assert!(common::calls(
        common::body_of(&lowered, "is_corner"),
        &ClassName::new("demo"),
        "Eq$Point$is_equal"
    ));
}

#[test]
fn a_derived_method_is_written_whether_anything_calls_it_or_not() {
    let source = concat!(
        "derive Eq for Point\n\n",
        "type Point = {\n    across: Int\n}\n"
    );

    let lowered = common::lowered(source);

    assert_eq!(common::methods_named(&lowered, "Eq$Point$is_equal"), 1);
}

#[test]
fn a_derived_eq_reads_each_field_of_both_values_in_declaration_order() {
    let lowered = common::lowered(POINT);

    assert_eq!(
        fields_read(common::body_of(&lowered, "Eq$Point$is_equal")),
        ["across", "across", "up", "up"]
    );
}

#[test]
fn a_derived_eq_compares_each_field_it_reads_and_answers_with_a_truth_value() {
    let lowered = common::lowered(POINT);
    let body = common::body_of(&lowered, "Eq$Point$is_equal");

    assert_eq!(comparisons(body), [Comparison::Equal, Comparison::Equal]);
    assert_eq!(answers(body), [true, false]);
}

/// How `Point` comes by its `Eq`, which a `Line` holding one reaches the same way either way.
const HOW_POINT_IS_COMPARED: [&str; 2] = [
    "derive Eq for Point",
    concat!(
        "instance Eq<Point> {\n    fn is_equal(one, other) -> Bool {\n",
        "        one.across == other.across\n    }\n}"
    ),
];

#[test]
fn a_derived_eq_reaches_a_held_type_s_own_instance_however_that_type_came_by_it() {
    for how in HOW_POINT_IS_COMPARED {
        let source = format!(
            concat!(
                "derive Eq for Line\n\n",
                "type Line = {{\n    from: Point\n}}\n\n{}\n\n",
                "type Point = {{\n    across: Int\n}}\n"
            ),
            how
        );

        let lowered = common::lowered(&source);

        assert!(
            common::calls(
                common::body_of(&lowered, "Eq$Line$is_equal"),
                &ClassName::new("demo"),
                "Eq$Point$is_equal"
            ),
            "{source}"
        );
    }
}

#[test]
fn a_derived_eq_of_variants_compares_the_tag_before_anything_a_variant_carries() {
    let source = concat!(
        "derive Eq for Payment\n\n",
        "type Payment =\n    | Pending\n    | Failed(String)\n"
    );

    let body = fetched(source, "Eq$Payment$is_equal");

    assert_eq!(fields_read(&body)[0..2], ["tag", "tag"]);
    assert_eq!(fields_read(&body)[3..], ["value0", "value0"]);
}

#[test]
fn a_variant_that_carries_nothing_is_answered_by_its_tag_alone() {
    let source = concat!(
        "derive Eq for Payment\n\n",
        "type Payment =\n    | Pending\n    | Settled\n"
    );

    let body = fetched(source, "Eq$Payment$is_equal");

    assert_eq!(fields_read(&body), ["tag", "tag"]);
}

/// The names of the fields a body reads, in the order it reads them.
fn fields_read(body: &Body) -> Vec<&str> {
    body.instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::GetField(field) => Some(field.name.as_str()),
            _ => None,
        })
        .collect()
}

/// Every comparison of two values a body makes, in the order it makes them.
fn comparisons(body: &Body) -> Vec<Comparison> {
    body.instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::CompareLongs(how) | Instruction::CompareIntegers(how) => Some(*how),
            _ => None,
        })
        .collect()
}

/// Every truth value a body pushes, which for a derived `Eq` is its two answers.
fn answers(body: &Body) -> Vec<bool> {
    body.instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::Boolean(value) => Some(*value),
            _ => None,
        })
        .collect()
}

/// The body `source` writes under `named`, taken out of the module it was lowered into.
fn fetched(source: &str, named: &str) -> Body {
    common::body_of(&common::lowered(source), named).clone()
}

/// The types a generated field is written at, each of which has `Eq` already.
const FIELDS: [&str; 3] = ["Int", "Bool", "String"];

#[hegel::test]
fn a_record_of_any_shape_derives_an_eq_that_reads_every_field_of_both_values(tc: TestCase) {
    let drawn: Vec<&str> = tc.draw(gs::vecs(gs::sampled_from(&FIELDS)).min_size(1));
    let lines = drawn
        .iter()
        .enumerate()
        .fold(String::new(), |mut lines, (position, at)| {
            let _ = writeln!(lines, "    value{position}: {at}");
            lines
        });
    let source = format!("derive Eq for Held\n\ntype Held = {{\n{lines}}}\n");

    let body = fetched(&source, "Eq$Held$is_equal");

    let wanted: Vec<String> = (0..drawn.len())
        .flat_map(|position| [format!("value{position}"), format!("value{position}")])
        .collect();
    assert_eq!(fields_read(&body), wanted, "{source}");
}
