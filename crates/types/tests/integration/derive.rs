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

/// A call of `of`'s one method over a module that derives `of` for the type `declared`.
fn deriving(of: &str, written: &str, declared: &str) -> String {
    let gives = gives_back(of);
    // A function giving back a `Bool` is named as the question it answers, which
    // `docs/specs/naming.md` holds every declaration to, whatever else it is doing here.
    let named = if gives == "Bool" { "is_read" } else { "read" };
    format!(
        concat!(
            "fn {}(one: Held, other: Held) -> {} {{\n    {}\n}}\n\n",
            "derive {} for Held\n\n{}\n"
        ),
        named, gives, written, of, declared
    )
}

/// What each standard trait's one method gives back, which a call of it is held to.
fn gives_back(of: &str) -> &'static str {
    match of {
        "Hash" => "Int",
        "Show" => "String",
        _ => "Bool",
    }
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

/// Each standard trait, with the call of its one method over two values named `one` and `other`.
const STANDARD: [(&str, &str); 4] = [
    ("Eq", "is_equal(one, other)"),
    ("Ord", "is_less(one, other)"),
    ("Hash", "hashed(one)"),
    ("Show", "shown(one)"),
];

#[test]
fn a_derive_of_each_standard_trait_is_the_instance_a_call_of_its_method_reaches() {
    for (of, written) in STANDARD {
        let source = deriving(of, written, &holding("Int"));

        assert_eq!(inferred_type(&source, written, 1), gives_back(of), "{of}");
    }
}

#[test]
fn a_type_deriving_one_trait_and_not_another_is_refused_only_where_the_other_is_asked() {
    let source = concat!(
        "fn is_read(one: Held, other: Held) -> Bool {\n    is_less(one, other)\n}\n\n",
        "derive Eq for Held\n\n",
        "type Held = {\n    id: Int\n}\n"
    );

    assert_eq!(refusal(source).message(), "`Held` has no instance of `Ord`");
}

#[test]
fn a_derive_naming_two_traits_writes_both_of_them() {
    let source = concat!(
        "fn is_read(one: Held, other: Held) -> Bool {\n    one == other && one < other\n}\n\n",
        "derive Eq, Ord for Held\n\n",
        "type Held = {\n    id: Int\n}\n"
    );

    assert_eq!(inferred_type(source, "one < other", 1), "Bool");
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

/// Whichever trait is derived, a record of fields that all have it derives it too.
#[hegel::test]
fn a_record_whose_every_field_has_the_trait_derives_it(tc: TestCase) {
    let (of, written) = tc.draw(gs::sampled_from(&STANDARD));
    let fields: Vec<&str> = tc.draw(gs::vecs(gs::sampled_from(&COMPARABLE)).min_size(1));
    let source = deriving(of, written, &record(&fields));

    assert_eq!(
        inferred_type(&source, written, 1),
        gives_back(of),
        "{source}"
    );
}

/// Whichever trait is derived, a field without it is refused naming the field and the trait.
#[hegel::test]
fn a_record_with_a_field_that_has_not_got_the_trait_is_refused_naming_both(tc: TestCase) {
    let (of, written) = tc.draw(gs::sampled_from(&STANDARD));
    let mut fields: Vec<&str> = tc.draw(gs::vecs(gs::sampled_from(&COMPARABLE)));
    let without: &str = tc.draw(gs::sampled_from(&INCOMPARABLE));
    let at: usize = tc.draw(gs::integers().min_value(0).max_value(fields.len()));
    fields.insert(at, without);
    let source = deriving(of, written, &record(&fields));

    assert_eq!(
        refusal(&source).message(),
        format!("`Held` derives `{of}`, and the `{without}` it holds as `value{at}` has none"),
        "{source}"
    );
}
