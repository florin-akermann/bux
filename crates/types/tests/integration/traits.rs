//! Typing a trait, an instance, and a constraint, which `docs/specs/traits.md` states.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ast::Span;

use crate::common::{inferred, inferred_type, refusal};

/// One trait, one instance, and a generic constrained by it, used at the type that has it.
const AREA: &str = concat!(
    "fn is_wide(shape: Square) -> Bool {\n    is_bigger(shape: shape, than: shape)\n}\n\n",
    "fn is_bigger<T: Area<T>>(shape: T, than: T) -> Bool {\n",
    "    covered(shape) > covered(than)\n}\n\n",
    "instance Area<Square> {\n    fn covered(shape: Square) -> Int {\n",
    "        shape.side * shape.side\n    }\n}\n\n",
    "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n\n",
    "type Square = {\n    side: Int\n}\n"
);

/// The type a use of the trait method written at `written` reached its instance at.
fn instance_at(source: &str, written: &str, occurrence: usize) -> String {
    let at = source
        .match_indices(written)
        .nth(occurrence - 1)
        .unwrap_or_else(|| panic!("{source:?} writes {written:?} {occurrence} times"))
        .0;
    inferred(source)
        .instance_at(Span::new(at, written.len()))
        .unwrap_or_else(|| panic!("{written:?} number {occurrence} reaches an instance"))
        .to_string()
}

#[test]
fn a_trait_method_is_typed_by_the_signature_its_trait_declares() {
    assert_eq!(inferred_type(AREA, "covered(shape)", 1), "Int");
}

#[test]
fn a_use_of_a_trait_method_reaches_the_instance_of_the_named_type_it_settled_on() {
    let source = concat!(
        "fn is_wide(shape: Square) -> Bool {\n    covered(shape) > 4\n}\n\n",
        "instance Area<Square> {\n    fn covered(shape: Square) -> Int {\n",
        "        shape.side * shape.side\n    }\n}\n\n",
        "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n\n",
        "type Square = {\n    side: Int\n}\n"
    );

    assert_eq!(instance_at(source, "covered", 1), "Square");
}

/// Inside a generic, the instance is the constraint the type parameter declares, which is what
/// specializing the body at each type then settles; `docs/specs/traits.md` states the three cases.
#[test]
fn a_use_inside_a_constrained_generic_reaches_the_type_parameter_it_is_written_over() {
    assert_eq!(instance_at(AREA, "covered", 1), "T");
}

#[test]
fn an_instance_method_takes_the_type_the_instance_is_for() {
    let source = concat!(
        "instance Eq<Point> {\n    fn is_equal(one, other) -> Bool {\n",
        "        one.across == other.across\n    }\n}\n\n",
        "type Point = {\n    across: Int\n}\n"
    );

    assert_eq!(inferred_type(source, "one", 2), "Point");
}

#[test]
fn a_trait_method_used_at_a_type_with_no_instance_is_refused() {
    let error = refusal(concat!(
        "fn is_wide(shape: Circle) -> Bool {\n    covered(shape) > 4\n}\n\n",
        "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n\n",
        "type Circle = {\n    radius: Int\n}\n"
    ));

    assert_eq!(error.message(), "`Circle` has no instance of `Area`");
}

#[test]
fn a_constrained_generic_used_at_a_type_with_no_instance_is_refused() {
    let error = refusal(concat!(
        "fn is_wide(shape: Circle) -> Bool {\n    is_bigger(shape: shape, than: shape)\n}\n\n",
        "fn is_bigger<T: Area<T>>(shape: T, than: T) -> Bool {\n",
        "    covered(shape) > covered(than)\n}\n\n",
        "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n\n",
        "type Circle = {\n    radius: Int\n}\n"
    ));

    assert_eq!(error.message(), "`Circle` has no instance of `Area`");
}

#[test]
fn a_trait_method_called_where_no_constraint_promises_it_is_refused() {
    let error = refusal(concat!(
        "fn is_wide<T>(shape: T) -> Bool {\n    covered(shape) > 4\n}\n\n",
        "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n"
    ));

    assert_eq!(error.message(), "`T` has no instance of `Area`");
}

#[test]
fn an_instance_method_is_held_to_the_signature_its_trait_gives_it() {
    let error = refusal(concat!(
        "instance Area<Square> {\n    fn covered(shape: Square) -> String {\n",
        "        \"four\"\n    }\n}\n\n",
        "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n\n",
        "type Square = {\n    side: Int\n}\n"
    ));

    assert_eq!(
        error.message(),
        "expected `(Square) -> Int`, found `(Square) -> String`"
    );
}

#[test]
fn a_comparison_reaches_the_instance_of_eq_the_type_has() {
    let source = concat!(
        "fn is_corner(point: Point) -> Bool {\n    point == point\n}\n\n",
        "instance Eq<Point> {\n    fn is_equal(one, other) -> Bool {\n",
        "        one.across == other.across\n    }\n}\n\n",
        "type Point = {\n    across: Int\n}\n"
    );

    assert_eq!(inferred_type(source, "point == point", 1), "Bool");
}

#[test]
fn a_comparison_of_a_type_with_no_instance_of_eq_is_refused_as_it_always_was() {
    let error = refusal(concat!(
        "fn is_corner(point: Point) -> Bool {\n    point == point\n}\n\n",
        "type Point = {\n    across: Int\n}\n"
    ));

    assert_eq!(
        error.message(),
        "`Point` has no `Eq`, so `==` is not written over it"
    );
}

#[test]
fn a_method_of_a_trait_that_gives_back_a_bool_asks_the_question_it_answers() {
    let error = refusal("trait Eqv<T> {\n    fn same(one: T, other: T) -> Bool\n}\n");

    assert_eq!(
        error.message(),
        "`same` gives back a `Bool`, so its name asks the question it answers"
    );
}

/// The types a trait can be given an instance for here, each with a value of it to compare.
///
#[test]
fn an_instance_is_for_a_whole_type_and_never_for_one_that_takes_arguments() {
    let error = refusal(
        "instance Eq<Option> {\n    fn is_equal(one, other) -> Bool {\n        true\n    }\n}\n",
    );

    assert_eq!(
        error.message(),
        "`Option` takes 1 type argument but 0 were given"
    );
}

#[test]
fn a_parameter_of_a_method_a_trait_declares_states_its_type() {
    let error = refusal("trait Shown<T> {\n    fn shown(one) -> Int\n}\n");

    assert_eq!(
        error.message(),
        "`one` states no type, and a signature is never inferred"
    );
}

/// `Bool`, `Int`, and `String` have the instances the compiler supplies, and `Point` has the one
/// the generated module writes; nothing else does, which is what the second property rests on.
const COMPARED: [(&str, &str); 4] = [
    ("Bool", "true"),
    ("Int", "1"),
    ("String", "\"one\""),
    ("Point", "Point { across: 1 }"),
];

/// The module both properties are drawn against, comparing two values of `of` through `Eq`.
///
/// `Point` is the one type it gives an instance to, so every other type it names is answered by
/// the compiler's instances or by nothing at all.
fn comparing(of: &str) -> String {
    format!(
        "fn is_same(one: {of}, other: {of}) -> Bool {{\n    is_equal(one, other)\n}}\n\n\
         instance Eq<Point> {{\n    fn is_equal(one, other) -> Bool {{\n        \
         one.across == other.across\n    }}\n}}\n\n\
         type Point = {{\n    across: Int\n}}\n"
    )
}

/// A trait method resolves to the instance of the type its use settled on, and to no other.
#[hegel::test]
fn a_trait_method_reaches_the_instance_of_the_type_it_is_used_at(tc: TestCase) {
    let (of, _) = tc.draw(gs::sampled_from(&COMPARED));
    let source = comparing(of);

    assert_eq!(instance_at(&source, "is_equal", 1), of);
}

/// A trait with no instance for a type is refused there, whatever else the module says.
#[hegel::test]
fn a_type_with_no_instance_is_refused_wherever_the_trait_is_asked_of_it(tc: TestCase) {
    let (of, _) = tc.draw(gs::sampled_from(&COMPARED));
    let source = format!(
        "fn is_matching() -> Bool {{\n    is_equal(Circle {{ radius: 1 }}, Circle {{ radius: 1 }})\n}}\n\n\
         {}\n\
         type Circle = {{\n    radius: Int\n}}\n",
        comparing(of)
    );

    let error = refusal(&source);

    assert_eq!(error.message(), "`Circle` has no instance of `Eq`");
}
