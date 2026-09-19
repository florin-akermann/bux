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
        shape("type UserId = UserId(Int64)"),
        ["type UserId", "  variant UserId", "    named-type Int64"]
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
