//! The refusals and the scopes of `docs/specs/traits.md`: a trait, an instance, and a constraint.

use lumen_resolver::{DefinitionKind, Namespace, Origin};

use crate::common::{meaning, refusal};

/// A trait of the module's own, with one instance for one of its types.
const AREA: &str = concat!(
    "fn is_wide(shape: Square) -> Bool {\n    covered(shape) > 4\n}\n\n",
    "instance Area<Square> {\n    fn covered(shape: Square) -> Int {\n",
    "        shape.side * shape.side\n    }\n}\n\n",
    "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n\n",
    "type Square = {\n    side: Int\n}\n"
);

#[test]
fn a_trait_names_itself_among_the_types() {
    let found = meaning(AREA, Namespace::Type, "Area", 1).expect("`Area` has a definition");

    assert_eq!(found.kind, DefinitionKind::Trait);
}

#[test]
fn a_trait_names_each_of_its_methods_among_the_values() {
    let found = meaning(AREA, Namespace::Value, "covered", 1).expect("`covered` has a definition");

    assert_eq!(found.kind, DefinitionKind::TraitMethod);
}

#[test]
fn every_use_of_a_trait_method_means_the_trait_that_declares_it() {
    let declared = meaning(AREA, Namespace::Value, "covered", 3).expect("the trait declares it");
    let called = meaning(AREA, Namespace::Value, "covered", 1).expect("the call means it");

    assert_eq!(called.origin, declared.origin);
}

#[test]
fn the_prelude_supplies_eq_and_the_method_it_declares() {
    let source = "fn is_same(left: Int, right: Int) -> Bool {\n    is_equal(left, right)\n}\n";
    let method = meaning(source, Namespace::Value, "is_equal", 1).expect("`is_equal` is in scope");

    assert_eq!(method.kind, DefinitionKind::TraitMethod);
    assert_eq!(method.origin, Origin::Prelude);
}

#[test]
fn a_module_may_not_declare_a_second_instance_of_one_trait_for_one_type() {
    let error = refusal(concat!(
        "instance Eq<Point> {\n    fn is_equal(one, other) -> Bool {\n        true\n    }\n}\n\n",
        "instance Eq<Point> {\n    fn is_equal(one, other) -> Bool {\n        false\n    }\n}\n\n",
        "type Point = {\n    across: Int\n}\n"
    ));

    assert_eq!(error.message(), "`Eq` already has an instance for `Point`");
}

#[test]
fn a_module_may_not_write_an_instance_the_compiler_already_supplies() {
    let error = refusal(
        "instance Eq<Int> {\n    fn is_equal(one, other) -> Bool {\n        true\n    }\n}\n",
    );

    assert_eq!(error.message(), "`Eq` already has an instance for `Int`");
}

#[test]
fn an_instance_writes_every_method_its_trait_declares() {
    let error = refusal(&around_against(
        "    fn covered(shape: T) -> Int\n    fn around(shape: T) -> Int\n",
    ));

    assert_eq!(
        error.message(),
        "`Area` declares `covered`, which this instance does not write"
    );
}

#[test]
fn an_instance_writes_no_method_its_trait_never_declared() {
    let error = refusal(&around_against("    fn covered(shape: T) -> Int\n"));

    assert_eq!(
        error.message(),
        "`Area` declares no `around` for this instance to write"
    );
}

#[test]
fn a_type_written_where_a_trait_belongs_is_refused() {
    let error = refusal(concat!(
        "instance Square<Square> {\n    fn covered(shape: Square) -> Int {\n",
        "        shape.side\n    }\n}\n\n",
        "type Square = {\n    side: Int\n}\n"
    ));

    assert_eq!(error.message(), "`Square` is not a trait");
}

#[test]
fn a_constraint_that_names_no_trait_is_refused() {
    let error = refusal(concat!(
        "fn kept<T: Square<T>>(value: T) -> T {\n    value\n}\n\n",
        "type Square = {\n    side: Int\n}\n"
    ));

    assert_eq!(error.message(), "`Square` is not a trait");
}

#[test]
fn a_trait_written_where_a_type_belongs_is_refused() {
    let error = refusal("fn compared(value: Eq) -> Eq {\n    value\n}\n");

    assert_eq!(error.message(), "`Eq` is a trait, not a type");
}

#[test]
fn a_module_may_not_declare_a_name_the_prelude_gives_a_trait() {
    let error = refusal("trait Eq<T> {\n    fn is_equal(one: T, other: T) -> Bool\n}\n");

    assert_eq!(error.message(), "`Eq` is already in scope here");
}

#[test]
fn a_trait_method_is_a_name_that_is_called_and_never_read() {
    let error = refusal("fn read() -> Int {\n    is_equal\n}\n");

    assert_eq!(
        error.message(),
        "`is_equal` is a function, so it is written as a call"
    );
}

#[test]
fn an_instance_writes_one_body_for_each_method_and_never_two() {
    let error = refusal(concat!(
        "instance Area<Square> {\n    fn covered(shape: Square) -> Int {\n",
        "        shape.side * shape.side\n    }\n\n",
        "    fn covered(shape: Square) -> Int {\n        4 * shape.side\n    }\n}\n\n",
        "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n\n",
        "type Square = {\n    side: Int\n}\n"
    ));

    assert_eq!(
        error.message(),
        "`Area` declares `covered` once, and this instance writes it twice"
    );
}

#[test]
fn a_trait_written_where_an_instance_names_its_type_is_refused() {
    let error = refusal(concat!(
        "instance Eq<Area> {\n    fn is_equal(one, other) -> Bool {\n        true\n    }\n}\n\n",
        "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n"
    ));

    assert_eq!(error.message(), "`Area` is a trait, not a type");
}

/// An instance writing `around` for a trait that declares `declared`, which is all that differs.
///
/// One of the two methods is written and the other is not, so which way the pair fails to line
/// up is the trait's to say: it declares `around` and something more, or it declares neither.
fn around_against(declared: &str) -> String {
    format!(
        concat!(
            "instance Area<Square> {{\n    fn around(shape: Square) -> Int {{\n",
            "        4 * shape.side\n    }}\n}}\n\n",
            "trait Area<T> {{\n{declared}}}\n\n",
            "type Square = {{\n    side: Int\n}}\n"
        ),
        declared = declared
    )
}
