//! What a derive gives a type, and what it holds a type to, per `docs/specs/derive.md`.

use std::fmt::Write as _;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_diagnostics::Code;

use crate::common::{inferred_type, refusal};

/// A comparison of two of a type, over a module that derives that type's `Eq`.
fn comparing(declared: &str) -> String {
    format!(
        concat!(
            "fn is_same(one: Held, other: Held) -> Bool {{\n    one == other\n}}\n\n",
            "derive Eq for Held\n\n{}\n"
        ),
        declared
    )
}

/// A record of one field of the written type, which a derive above it is read against.
fn holding(written: &str) -> String {
    format!("type Held = {{\n    value: {written}\n}}")
}

#[test]
fn a_derived_eq_is_the_instance_a_comparison_of_that_type_reaches() {
    let source = comparing(&holding("Int"));

    assert_eq!(inferred_type(&source, "one == other", 1), "Bool");
}

#[test]
fn a_type_with_a_derive_is_compared_wherever_one_with_a_written_instance_would_be() {
    let source = comparing(&holding("String"));

    assert_eq!(inferred_type(&source, "one == other", 1), "Bool");
}

#[test]
fn a_type_deriving_eq_that_holds_a_type_with_none_is_refused_naming_what_it_holds() {
    let error = refusal(&comparing(&holding("List<Int>")));

    assert_eq!(
        error.message(),
        "`Held` derives `Eq`, and the `List<Int>` it holds as `value` has none"
    );
    assert_eq!(error.diagnostic().code(), Code::HeldTypeHasNoInstance);
}

#[test]
fn a_type_deriving_eq_that_holds_a_unit_is_refused_because_nothing_compares_two() {
    let error = refusal(&comparing("type Held = {\n    value: ()\n}"));

    assert_eq!(
        error.message(),
        "`Held` derives `Eq`, and the `()` it holds as `value` has none"
    );
}

#[test]
fn a_type_deriving_eq_may_hold_a_type_that_derives_it_further_down_the_file() {
    let source = concat!(
        "fn is_same(one: Held, other: Held) -> Bool {\n    one == other\n}\n\n",
        "derive Eq for Held\n\n",
        "type Held = {\n    inner: Inner\n}\n\n",
        "derive Eq for Inner\n\n",
        "type Inner = {\n    id: Int\n}\n"
    );

    assert_eq!(inferred_type(source, "one == other", 1), "Bool");
}

#[test]
fn a_variant_deriving_eq_names_the_variant_and_the_place_of_what_it_carries() {
    let source = comparing("type Held =\n    | Empty\n    | Full(List<Int>)");

    assert_eq!(
        refusal(&source).message(),
        "`Held` derives `Eq`, and the `List<Int>` it holds as `Full.0` has none"
    );
}

#[test]
fn a_comparison_of_a_type_with_no_eq_says_the_derive_that_would_give_it_one() {
    let source = concat!(
        "fn is_same(one: Held, other: Held) -> Bool {\n    one == other\n}\n\n",
        "type Held = {\n    id: Int\n}\n"
    );

    assert_eq!(
        refusal(source).help(),
        "`Held` gets one by deriving it: write `derive Eq for Held`"
    );
}

/// The types a generated field is written at, each of which has `Eq` and needs no declaration.
const COMPARABLE: [&str; 3] = ["Int", "Bool", "String"];

/// The types a generated field is written at that have no `Eq`, which a derive is refused for.
const INCOMPARABLE: [&str; 3] = ["List<Int>", "()", "Option<Int>"];

/// A record of the written fields, named `value0`, `value1`, and so on.
fn record(fields: &[&str]) -> String {
    let lines = fields
        .iter()
        .enumerate()
        .fold(String::new(), |mut written, (position, at)| {
            let _ = writeln!(written, "    value{position}: {at}");
            written
        });
    format!("type Held = {{\n{lines}}}")
}

#[hegel::test]
fn a_record_whose_every_field_has_eq_derives_it(tc: TestCase) {
    let fields: Vec<&str> = tc.draw(gs::vecs(gs::sampled_from(&COMPARABLE)).min_size(1));
    let source = comparing(&record(&fields));

    assert_eq!(
        inferred_type(&source, "one == other", 1),
        "Bool",
        "{source}"
    );
}

#[hegel::test]
fn a_record_with_one_field_that_has_no_eq_is_refused_naming_that_field(tc: TestCase) {
    let mut fields: Vec<&str> = tc.draw(gs::vecs(gs::sampled_from(&COMPARABLE)));
    let without: &str = tc.draw(gs::sampled_from(&INCOMPARABLE));
    let at: usize = tc.draw(gs::integers().min_value(0).max_value(fields.len()));
    fields.insert(at, without);
    let source = comparing(&record(&fields));

    assert_eq!(
        refusal(&source).message(),
        format!("`Held` derives `Eq`, and the `{without}` it holds as `value{at}` has none"),
        "{source}"
    );
}
