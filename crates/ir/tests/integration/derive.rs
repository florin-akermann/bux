//! What a derive writes, which `docs/specs/derive.md` makes the instance an author would have.

use std::fmt::Write as _;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{Body, ClassName, Comparison, Descriptor, Instruction};

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
fn a_variant_that_carries_nothing_is_answered_by_its_tag_alone() {
    let source = concat!(
        "derive Eq for Payment\n\n",
        "type Payment =\n    | Pending\n    | Settled\n"
    );

    let body = fetched(source, "Eq$Payment$is_equal");

    assert_eq!(fields_read(&body), ["tag", "tag"]);
}

/// A module deriving all four standard traits for a record of two `Int` fields.
const ALL_FOUR: &str = concat!(
    "derive Eq, Ord, Hash, Show for Point\n\n",
    "type Point = {\n    across: Int\n    up: Int\n}\n"
);

/// Each standard trait, with the method a derive of it writes.
const STANDARD: [(&str, &str); 4] = [
    ("Eq", "Eq$Point$is_equal"),
    ("Ord", "Ord$Point$is_less"),
    ("Hash", "Hash$Point$hashed"),
    ("Show", "Show$Point$shown"),
];

#[test]
fn a_derive_naming_four_traits_writes_the_method_of_each_one() {
    let lowered = common::lowered(ALL_FOUR);

    for (of, method) in STANDARD {
        assert_eq!(common::methods_named(&lowered, method), 1, "{of}");
    }
}

#[test]
fn a_derived_ord_reads_each_field_of_both_values_both_ways_round_in_declaration_order() {
    let body = fetched(ALL_FOUR, "Ord$Point$is_less");

    assert_eq!(
        fields_read(&body),
        [
            "across", "across", "across", "across", "up", "up", "up", "up"
        ]
    );
    assert_eq!(answers(&body), [true, false]);
}

#[test]
fn a_derived_hash_reads_each_field_once_in_declaration_order_and_gives_a_whole_number() {
    let body = fetched(ALL_FOUR, "Hash$Point$hashed");

    assert_eq!(fields_read(&body), ["across", "up"]);
    assert_eq!(
        body.instructions.last(),
        Some(&Instruction::Return(Some(Descriptor::Long)))
    );
}

/// What each derive of `Payment` reads, in order, where `Payment` is `Pending | Failed(String)`.
///
/// Every one of them reads the tag before what a variant carries, because which variant a value
/// is decides both what the answer is and which block reads anything at all.
const OVER_VARIANTS: [(&str, &[&str]); 4] = [
    (
        "Eq$Payment$is_equal",
        &["tag", "tag", "tag", "value0", "value0"],
    ),
    (
        "Ord$Payment$is_less",
        &[
            "tag", "tag", "tag", "tag", "tag", "value0", "value0", "value0", "value0",
        ],
    ),
    ("Hash$Payment$hashed", &["tag", "tag", "value0"]),
    ("Show$Payment$shown", &["tag", "value0"]),
];

#[test]
fn a_derive_over_variants_reads_the_tag_before_anything_a_variant_carries() {
    let source = concat!(
        "derive Eq, Ord, Hash, Show for Payment\n\n",
        "type Payment =\n    | Pending\n    | Failed(String)\n"
    );

    for (method, reads) in OVER_VARIANTS {
        let body = fetched(source, method);

        assert_eq!(fields_read(&body), reads, "{method}");
    }
}

#[test]
fn a_derived_show_writes_the_record_as_the_source_that_builds_it() {
    let body = fetched(ALL_FOUR, "Show$Point$shown");

    assert_eq!(texts(&body), ["Point {", " across: ", ", up: ", " }"]);
}

#[test]
fn a_derived_show_writes_a_variant_by_its_own_name_and_whatever_it_carries() {
    let source = concat!(
        "derive Show for Payment\n\n",
        "type Payment =\n    | Pending\n    | Failed(String)\n",
        "    | Sent {\n        note: String\n    }\n"
    );

    let body = fetched(source, "Show$Payment$shown");

    assert_eq!(
        texts(&body),
        ["Pending", "Failed(", ")", "Sent {", " note: ", " }"]
    );
}

#[test]
fn a_record_that_holds_nothing_is_shown_as_its_name_and_two_braces() {
    let source = "derive Show for Empty\n\ntype Empty = {\n}\n";

    let body = fetched(source, "Show$Empty$shown");

    assert_eq!(texts(&body), ["Empty {", "}"]);
}

#[test]
fn a_variant_declaring_braces_and_no_field_is_shown_with_those_braces() {
    let source = concat!(
        "derive Show for Payment\n\n",
        "type Payment =\n    | Sent {\n    }\n    | Pending\n"
    );

    let body = fetched(source, "Show$Payment$shown");

    assert_eq!(texts(&body), ["Sent {", "}", "Pending"]);
}

/// Every piece of text a body writes, in the order it writes them.
fn texts(body: &Body) -> Vec<&str> {
    body.instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::Text(written) => Some(written.as_str()),
            _ => None,
        })
        .collect()
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

/// Whichever trait is derived, a record of any shape reads every field in declaration order.
///
/// How many times each is read is the trait's own: `Eq` reads both values of it, `Ord` reads them
/// both ways round, and `Hash` and `Show` read the one value they are handed.
#[hegel::test]
fn a_record_of_any_shape_derives_a_body_that_reads_every_field_in_order(tc: TestCase) {
    let (of, method) = tc.draw(gs::sampled_from(&STANDARD));
    let drawn: Vec<&str> = tc.draw(gs::vecs(gs::sampled_from(&FIELDS)).min_size(1));
    let lines = drawn
        .iter()
        .enumerate()
        .fold(String::new(), |mut lines, (position, at)| {
            let _ = writeln!(lines, "    value{position}: {at}");
            lines
        });
    let source = format!("derive {of} for Point\n\ntype Point = {{\n{lines}}}\n");

    let body = fetched(&source, method);

    let each = times_read(of);
    let wanted: Vec<String> = (0..drawn.len())
        .flat_map(|position| vec![format!("value{position}"); each])
        .collect();
    assert_eq!(fields_read(&body), wanted, "{source}");
}

/// How often a derive of `of` reads one field the type holds.
fn times_read(of: &str) -> usize {
    match of {
        "Eq" => 2,
        "Ord" => 4,
        _ => 1,
    }
}
