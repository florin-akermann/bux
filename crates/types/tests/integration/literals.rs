//! A whole-number literal takes the type its context expects, which `docs/specs/literals.md` says.
//!
//! The type is a variable standing for a whole number, so it settles on whatever the code around
//! it says, on a type with an instance of `IntegerLiteral` and on nothing else, and on `Int` where
//! nothing says anything. What it settles on then says which numbers it holds.

use hegel::TestCase;
use hegel::generators as gs;

use crate::common::{inferred_type, refusal};

/// A declared type that takes a whole number, beside the instance saying which ones it holds.
const INT32: &str = concat!(
    "instance IntegerLiteral<Int32> {\n",
    "    fn lowest() -> Int {\n        -2147483648\n    }\n\n",
    "    fn highest() -> Int {\n        2147483647\n    }\n\n",
    "    fn from_literal(literal: Int) -> Int32 {\n        Int32(literal)\n    }\n}\n\n",
    "type Int32 = Int32(Int)\n"
);

/// The whole numbers an `Int32` holds, and the ones just outside it on either side.
const AT_THE_EDGE: [(&str, bool); 6] = [
    ("-2147483649", false),
    ("-2147483648", true),
    ("-1", true),
    ("0", true),
    ("2147483647", true),
    ("2147483648", false),
];

#[test]
fn a_literal_in_a_declared_result_is_the_type_that_result_declares() {
    let source = format!("fn counted() -> Int32 {{\n    42\n}}\n\n{INT32}");

    assert_eq!(inferred_type(&source, "42", 1), "Int32");
}

#[test]
fn a_literal_in_an_argument_is_the_type_the_parameter_declares() {
    let source = format!(
        "fn main() -> Int32 {{\n    passed(count: 42)\n}}\n\nfn passed(count: Int32) -> Int32 {{\n    count\n}}\n\n{INT32}"
    );

    assert_eq!(inferred_type(&source, "42", 1), "Int32");
}

#[test]
fn a_literal_nothing_settles_is_an_int() {
    assert_eq!(
        inferred_type("fn counted() -> Int {\n    42\n}\n", "42", 1),
        "Int"
    );
}

#[test]
fn two_literals_added_together_are_one_type_and_settle_together() {
    let source = format!(
        "fn counted() -> Int32 {{\n    twice(one: 20, other: 22)\n}}\n\nfn twice(one: Int32, other: Int32) -> Int32 {{\n    one\n}}\n\n{INT32}"
    );

    assert_eq!(inferred_type(&source, "20", 1), "Int32");
    assert_eq!(inferred_type(&source, "22", 1), "Int32");
}

#[test]
fn a_name_bound_to_a_literal_still_takes_the_type_its_use_expects() {
    let source = format!("fn counted() -> Int32 {{\n    count := 5\n    count\n}}\n{INT32}");

    assert_eq!(inferred_type(&source, "5", 1), "Int32");
}

#[test]
fn a_whole_number_left_where_nothing_takes_it_is_still_a_discarded_int() {
    let refused = refusal("fn counted() -> Int {\n    5\n    6\n}\n");

    assert_eq!(refused.message(), "`Int` is left here and nothing takes it");
}

#[test]
fn from_literal_is_an_ordinary_trait_method_a_program_may_call_by_name() {
    assert_eq!(
        inferred_type(
            "fn counted() -> Int {\n    from_literal(7)\n}\n",
            "from_literal(7)",
            1
        ),
        "Int"
    );
}

#[test]
fn a_bound_names_no_instance_a_call_of_it_could_mean_so_a_call_is_refused() {
    let source = format!("fn counted() -> Int {{\n    lowest()\n}}\n{INT32}");

    assert_eq!(
        refusal(&source).message(),
        "`_` has no instance of `IntegerLiteral`"
    );
}

#[test]
fn a_whole_number_written_as_a_pattern_is_an_int_rather_than_a_literal() {
    let source = format!(
        "fn counted(count: Int32) -> Int {{\n    match count {{\n        5 => 1\n    }}\n}}\n{INT32}"
    );

    let refused = refusal(&source);
    assert_eq!(refused.message(), "expected `Int32`, found `Int`");
    assert_eq!(refused.span().text(&source), "5");
}

#[test]
fn a_literal_outside_the_range_its_instance_states_is_refused() {
    let source = format!("fn counted() -> Int32 {{\n    5000000000\n}}\n\n{INT32}");

    let refused = refusal(&source);
    assert_eq!(
        refused.message(),
        "`5000000000` does not fit `Int32`, which holds `-2147483648` to `2147483647`"
    );
    assert_eq!(refused.span().text(&source), "5000000000");
}

#[test]
fn a_literal_at_a_type_that_takes_none_reads_as_the_int_it_would_have_been() {
    let refused = refusal("fn is_open() -> Bool {\n    1\n}\n");

    assert_eq!(refused.message(), "expected `Bool`, found `Int`");
}

#[test]
fn a_literal_at_a_type_parameter_is_refused_however_that_parameter_is_constrained() {
    let source = format!("fn counted<T: IntegerLiteral<T>>() -> T {{\n    1\n}}\n\n{INT32}");

    assert_eq!(refusal(&source).message(), "expected `T`, found `Int`");
}

#[test]
fn a_literal_beside_an_operator_takes_the_type_the_other_side_of_it_is() {
    let adds = "instance Add<Int32> {\n    fn add(one: Int32, other: Int32) -> Int32 {\n        one\n    }\n}\n";
    let source =
        format!("fn added(count: Int32) -> Int32 {{\n    count + 9\n}}\n\n{adds}\n{INT32}");

    assert_eq!(inferred_type(&source, "9", 1), "Int32");
}

#[test]
fn a_literal_beside_an_operator_over_a_type_that_takes_none_is_refused_at_the_number() {
    let source = concat!(
        "fn added(count: Money) -> Money {\n    count + 2\n}\n\n",
        "instance Add<Money> {\n    fn add(one: Money, other: Money) -> Money {\n",
        "        one\n    }\n}\n\ntype Money = {\n    cents: Int\n}\n"
    );

    let refused = refusal(source);
    assert_eq!(refused.message(), "expected `Money`, found `Int`");
    assert_eq!(refused.span().text(source), "2");
}

#[test]
fn a_bound_that_is_not_one_whole_number_is_refused() {
    let source = INT32.replace("-2147483648", "0 - 2147483648");

    assert_eq!(
        refusal(&source).message(),
        "`lowest` of `Int32` is read rather than run, so it is one whole number"
    );
}

/// A literal nothing settles is an `Int`, whatever the module around it declares.
#[hegel::test]
fn a_literal_nothing_settles_is_an_int_whatever_is_declared_beside_it(tc: TestCase) {
    let value: i64 = tc.draw(gs::integers().min_value(-1_000_000).max_value(1_000_000));
    let source = format!("fn counted() -> Int {{\n    {value}\n}}\n\n{INT32}");

    assert_eq!(inferred_type(&source, &value.to_string(), 1), "Int");
}

/// A literal inside the range its instance states is accepted, and one outside it is refused.
#[hegel::test]
fn a_literal_is_accepted_exactly_where_the_range_its_instance_states_holds_it(tc: TestCase) {
    let (written, fits) = tc.draw(gs::sampled_from(&AT_THE_EDGE));
    let source = format!("fn counted() -> Int32 {{\n    {written}\n}}\n\n{INT32}");

    assert_eq!(inferred_type_or_refusal(&source, written), fits);
}

/// A whole number written where an `Int` is expected infers exactly as it did before.
#[hegel::test]
fn a_whole_number_where_an_int_is_expected_is_an_int(tc: TestCase) {
    let value: i64 = tc.draw(gs::integers().min_value(0).max_value(i64::MAX));
    let source = format!("fn counted(number: Int) -> Int {{\n    number + {value}\n}}\n");

    assert_eq!(inferred_type(&source, &value.to_string(), 1), "Int");
}

/// Whether `source` is accepted, which for these modules is whether the literal fits.
fn inferred_type_or_refusal(source: &str, written: &str) -> bool {
    let parsed = lumen_parser::parse(source).expect("the module parses");
    let resolved = lumen_resolver::resolve(parsed).expect("every name of the module resolves");
    match lumen_types::check(resolved, &lumen_types::Imported::default()) {
        Ok(_) => true,
        Err(refused) => {
            assert_eq!(refused.span().text(source), written);
            false
        }
    }
}
