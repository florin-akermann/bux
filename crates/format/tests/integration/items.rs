//! Items: how canonical form writes an import, a type, and a function.

use crate::common::formatted;

#[test]
fn an_import_is_one_line() {
    assert_eq!(formatted("import   io\n"), "import io\n");
}

#[test]
fn a_blank_line_separates_two_items_and_only_one_does() {
    assert_eq!(
        formatted("import io\n\n\n\nimport files\n"),
        "import io\n\nimport files\n"
    );
    assert_eq!(
        formatted("import io\nimport files\n"),
        "import io\n\nimport files\n"
    );
}

#[test]
fn a_record_type_writes_one_field_per_line() {
    assert_eq!(
        formatted("type User = {\nid: UserId\nactive: Bool\n}\n"),
        "type User = {\n    id: UserId\n    active: Bool\n}\n"
    );
}

#[test]
fn a_record_type_with_no_fields_still_spans_two_lines() {
    assert_eq!(formatted("type Empty = {}\n"), "type Empty = {\n}\n");
}

#[test]
fn a_single_variant_is_written_on_the_type_line_without_a_bar() {
    assert_eq!(
        formatted("type UserId =\n| UserId(Int64)\n"),
        "type UserId = UserId(Int64)\n"
    );
}

#[test]
fn a_variant_list_of_two_or_more_writes_one_bar_per_line() {
    assert_eq!(
        formatted("type Payment = | Pending | Failed(String)\n"),
        "type Payment =\n    | Pending\n    | Failed(String)\n"
    );
}

#[test]
fn a_variants_record_payload_closes_at_the_level_of_its_bar() {
    assert_eq!(
        formatted("type Payment = | Pending | Authorized { id: String }\n"),
        concat!(
            "type Payment =\n",
            "    | Pending\n",
            "    | Authorized {\n",
            "        id: String\n",
            "    }\n",
        )
    );
}

#[test]
fn a_function_writes_its_signature_on_one_line() {
    assert_eq!(
        formatted("fn add(a:Int,b:Int)->Int{\na+b\n}\n"),
        "fn add(a: Int, b: Int) -> Int {\n    a + b\n}\n"
    );
}

#[test]
fn a_function_may_be_generic_and_may_leave_a_parameter_to_inference() {
    assert_eq!(
        formatted("fn first<T,E>(items:List<T>,fallback)->Option<T>{\nfallback\n}\n"),
        "fn first<T, E>(items: List<T>, fallback) -> Option<T> {\n    fallback\n}\n"
    );
}

#[test]
fn a_function_without_a_result_type_writes_none() {
    assert_eq!(formatted("fn f(){\n}\n"), "fn f() {\n}\n");
}

#[test]
fn an_empty_file_formats_to_nothing() {
    assert_eq!(formatted(""), "");
}
