//! Imports, type declarations, and functions: what a source file holds.

use crate::common::shape;

#[test]
fn an_import_names_one_module() {
    assert_eq!(shape("import io"), ["import io"]);
}

#[test]
fn a_file_holds_its_items_in_source_order() {
    assert_eq!(
        shape("import io\n\nimport files\n"),
        ["import io", "import files"]
    );
}

#[test]
fn a_newtype_is_a_single_variant_carrying_one_type() {
    assert_eq!(
        shape("type UserId = UserId(Int)"),
        ["type UserId", "  variant UserId", "    named-type Int"]
    );
}

#[test]
fn a_record_type_declares_one_field_per_line() {
    assert_eq!(
        shape("type User = {\n    id: UserId\n    active: Bool\n}"),
        [
            "type User",
            "  field id",
            "    named-type UserId",
            "  field active",
            "    named-type Bool",
        ]
    );
}

#[test]
fn a_sum_type_declares_one_variant_per_line() {
    assert_eq!(
        shape("type Payment =\n    | Pending\n    | Failed(String)\n"),
        [
            "type Payment",
            "  variant Pending",
            "  variant Failed",
            "    named-type String",
        ]
    );
}

#[test]
fn a_variant_may_carry_named_fields() {
    assert_eq!(
        shape("type Payment =\n    | Authorized {\n        id: String\n    }\n"),
        [
            "type Payment",
            "  variant Authorized",
            "    field id",
            "      named-type String",
        ]
    );
}

#[test]
fn a_type_declaration_may_be_generic() {
    assert_eq!(
        shape("type Result<T, E> =\n    | Ok(T)\n    | Err(E)\n"),
        [
            "type Result",
            "  type-parameter T",
            "  type-parameter E",
            "  variant Ok",
            "    named-type T",
            "  variant Err",
            "    named-type E",
        ]
    );
}

#[test]
fn a_type_argument_may_itself_be_generic() {
    assert_eq!(
        shape("fn f(x: Result<List<User>, E>) {\n}"),
        [
            "function f",
            "  parameter x",
            "    named-type Result",
            "      named-type List",
            "        named-type User",
            "      named-type E",
            "  block",
        ]
    );
}

#[test]
fn a_field_whose_type_ends_in_a_closing_angle_bracket_still_ends_its_line() {
    assert_eq!(
        shape("type P = {\n    a: List<List<Int>>\n    b: Int\n}"),
        [
            "type P",
            "  field a",
            "    named-type List",
            "      named-type List",
            "        named-type Int",
            "  field b",
            "    named-type Int",
        ]
    );
}

#[test]
fn a_function_declares_its_parameters_and_its_result() {
    assert_eq!(
        shape("fn add(a: Int, b: Int) -> Int {\n    a + b\n}"),
        [
            "function add",
            "  parameter a",
            "    named-type Int",
            "  parameter b",
            "    named-type Int",
            "  result",
            "    named-type Int",
            "  block",
            "    binary Add",
            "      name a",
            "      name b",
        ]
    );
}

#[test]
fn a_parameter_may_be_left_to_inference() {
    assert_eq!(
        shape("fn identity(x) {\n    x\n}"),
        [
            "function identity",
            "  parameter x",
            "  block",
            "    name x"
        ]
    );
}

#[test]
fn a_function_may_be_generic() {
    assert_eq!(
        shape("fn first<T>(items: List<T>) -> Option<T> {\n}"),
        [
            "function first",
            "  type-parameter T",
            "  parameter items",
            "    named-type List",
            "      named-type T",
            "  result",
            "    named-type Option",
            "      named-type T",
            "  block",
        ]
    );
}

#[test]
fn a_result_type_may_be_unit() {
    assert_eq!(
        shape("fn save() -> Result<(), E> {\n}"),
        [
            "function save",
            "  result",
            "    named-type Result",
            "      unit-type",
            "      named-type E",
            "  block",
        ]
    );
}

#[test]
fn a_parameter_list_may_span_lines() {
    assert_eq!(
        shape("fn contains(\n    items: List<T>,\n    value: T\n) -> Bool {\n}"),
        [
            "function contains",
            "  parameter items",
            "    named-type List",
            "      named-type T",
            "  parameter value",
            "    named-type T",
            "  result",
            "    named-type Bool",
            "  block",
        ]
    );
}

#[test]
fn a_comment_is_not_part_of_the_tree() {
    assert_eq!(
        shape("// a module\n\nimport io // and its comment\n\n// done\n"),
        ["import io"]
    );
}

#[test]
fn an_empty_file_is_a_program_with_no_items() {
    assert_eq!(shape(""), [] as [String; 0]);
    assert_eq!(shape("\n\n// nothing here\n"), [] as [String; 0]);
}

#[test]
fn a_trait_declares_one_type_parameter_and_one_signature_per_line() {
    assert_eq!(
        shape("trait Eq<T> {\n    fn is_equal(one: T, other: T) -> Bool\n}"),
        [
            "trait Eq",
            "  type-parameter T",
            "  signature is_equal",
            "    parameter one",
            "      named-type T",
            "    parameter other",
            "      named-type T",
            "    result",
            "      named-type Bool",
        ]
    );
}

#[test]
fn a_trait_declares_as_many_signatures_as_it_writes() {
    assert_eq!(
        shape(
            "trait Show<T> {\n    fn shown(value: T) -> String\n    fn width(value: T) -> Int\n}"
        ),
        [
            "trait Show",
            "  type-parameter T",
            "  signature shown",
            "    parameter value",
            "      named-type T",
            "    result",
            "      named-type String",
            "  signature width",
            "    parameter value",
            "      named-type T",
            "    result",
            "      named-type Int",
        ]
    );
}

#[test]
fn an_instance_names_a_trait_and_the_type_it_is_for() {
    assert_eq!(
        shape(
            "instance Eq<Point> {\n    fn is_equal(one, other) -> Bool {\n        true\n    }\n}"
        ),
        [
            "instance Eq<Point>",
            "  function is_equal",
            "    parameter one",
            "    parameter other",
            "    result",
            "      named-type Bool",
            "    block",
            "      bool true",
        ]
    );
}

#[test]
fn a_derive_names_one_trait_and_the_type_the_compiler_writes_it_for() {
    assert_eq!(shape("derive Eq for User"), ["derive Eq for User"]);
}

#[test]
fn a_derive_names_every_trait_it_lists_in_the_order_it_lists_them() {
    assert_eq!(
        shape("derive Eq, Ord, Hash for Payment"),
        ["derive Eq, Ord, Hash for Payment"]
    );
}

#[test]
fn a_type_parameter_is_written_with_the_trait_it_is_constrained_by() {
    assert_eq!(
        shape("fn has_value<T: Eq<T>>(value: T) -> Bool {\n    true\n}"),
        [
            "function has_value",
            "  type-parameter T",
            "    constraint Eq",
            "      named-type T",
            "  parameter value",
            "    named-type T",
            "  result",
            "    named-type Bool",
            "  block",
            "    bool true",
        ]
    );
}

#[test]
fn a_type_parameter_without_a_constraint_is_written_as_it_always_was() {
    assert_eq!(
        shape("fn kept<T>(value: T) -> T {\n    value\n}"),
        [
            "function kept",
            "  type-parameter T",
            "  parameter value",
            "    named-type T",
            "  result",
            "    named-type T",
            "  block",
            "    name value",
        ]
    );
}

#[test]
fn a_type_of_another_module_is_written_through_the_name_it_is_reached_by() {
    assert_eq!(
        shape("type Holder = Holder(demo.User)"),
        [
            "type Holder",
            "  variant Holder",
            "    named-type demo.User"
        ]
    );
}

#[test]
fn a_type_of_another_module_takes_its_arguments_after_the_whole_name() {
    assert_eq!(
        shape("type Holder = Holder(demo.Held<Int>)"),
        [
            "type Holder",
            "  variant Holder",
            "    named-type demo.Held",
            "      named-type Int",
        ]
    );
}

#[test]
fn an_extern_says_its_member_gives_an_int_by_writing_the_width_after_the_kind() {
    assert_eq!(
        shape("extern method int length(text: String) -> Int = \"length\""),
        [
            "extern method int length",
            "  java length",
            "  parameter text",
            "    named-type String",
            "  result",
            "    named-type Int",
        ]
    );
}

#[test]
fn int_is_the_width_only_where_a_name_follows_it_and_is_an_ordinary_name_otherwise() {
    assert_eq!(
        shape("extern static int(text: String) -> Int = \"java.lang.String.length\""),
        [
            "extern static int",
            "  java java.lang.String.length",
            "  parameter text",
            "    named-type String",
            "  result",
            "    named-type Int",
        ]
    );
}

#[test]
fn an_extern_says_its_member_gives_a_char_the_same_way_it_says_it_gives_an_int() {
    assert_eq!(
        shape("extern method char at(text: String, int index: Int) -> Option<Int> = \"charAt\""),
        [
            "extern method char at",
            "  java charAt",
            "  parameter text",
            "    named-type String",
            "  parameter int index",
            "    named-type Int",
            "  result",
            "    named-type Option",
            "      named-type Int",
        ]
    );
}

#[test]
fn a_parameter_named_int_is_a_name_and_not_the_width_the_member_takes() {
    assert_eq!(
        shape("extern method held(text: String, int: Int) -> Option<Int> = \"charAt\""),
        [
            "extern method held",
            "  java charAt",
            "  parameter text",
            "    named-type String",
            "  parameter int",
            "    named-type Int",
            "  result",
            "    named-type Option",
            "      named-type Int",
        ]
    );
}
