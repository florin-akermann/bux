//! What a trait, an instance, and a constraint become, which `docs/specs/traits.md` states.

use lumen_ir::ClassName;

use crate::common;

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

#[test]
fn an_instance_method_is_named_for_its_trait_its_type_and_itself() {
    let lowered = common::lowered(AREA);

    assert!(common::has_method(&lowered, "Area$Square$covered"));
    assert!(
        !common::has_method(&lowered, "covered"),
        "an instance declares no name, so nothing is written under the method's own"
    );
}

#[test]
fn an_instance_method_is_written_whether_anything_calls_it_or_not() {
    let source = concat!(
        "instance Area<Square> {\n    fn covered(shape: Square) -> Int {\n",
        "        shape.side * shape.side\n    }\n}\n\n",
        "trait Area<T> {\n    fn covered(shape: T) -> Int\n}\n\n",
        "type Square = {\n    side: Int\n}\n"
    );

    let lowered = common::lowered(source);

    assert_eq!(common::methods_named(&lowered, "Area$Square$covered"), 1);
}

#[test]
fn a_generic_specialized_at_a_type_calls_the_instance_that_type_has() {
    let lowered = common::lowered(AREA);

    let specialized = common::body_of(&lowered, "is_bigger$Square");

    assert!(common::calls(
        specialized,
        &ClassName::new("demo"),
        "Area$Square$covered"
    ));
}

#[test]
fn a_comparison_of_a_type_with_an_instance_of_eq_calls_that_instance() {
    let source = concat!(
        "fn is_corner(point: Point) -> Bool {\n    point == point\n}\n\n",
        "instance Eq<Point> {\n    fn is_equal(one, other) -> Bool {\n",
        "        one.across == other.across\n    }\n}\n\n",
        "type Point = {\n    across: Int\n}\n"
    );

    let lowered = common::lowered(source);

    assert!(common::calls(
        common::body_of(&lowered, "is_corner"),
        &ClassName::new("demo"),
        "Eq$Point$is_equal"
    ));
}

#[test]
fn a_call_of_an_instance_over_a_type_the_jvm_holds_is_written_out_where_it_stands() {
    let source = "fn is_same(left: Int, right: Int) -> Bool {\n    is_equal(left, right)\n}\n";

    let lowered = common::lowered(source);

    assert!(
        !common::calls(
            common::body_of(&lowered, "is_same"),
            &ClassName::new("demo"),
            "Eq$Int$is_equal"
        ),
        "an instance over a type the JVM holds has no method to call"
    );
    assert_eq!(common::methods_named(&lowered, "is_same"), 1);
}
