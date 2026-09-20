//! `docs/specs/calls.md`: the name after a dot is a name in scope unless a module is before it.

use lumen_resolver::DefinitionKind;
use lumen_resolver::Namespace::Value;

use crate::common::{meaning, refusal, resolved};

/// A module declaring `held`, with `body` as the body of the function that calls it.
fn calling_held(body: &str) -> String {
    format!(
        "fn go() -> Int {{\n{body}}}\n\nfn held(value: Int, more: Int) -> Int {{\n    value + more\n}}\n"
    )
}

#[test]
fn the_name_after_the_dot_means_the_function_this_module_declares() {
    let source = calling_held("    count := 1\n    count.held(2)\n");

    let called = meaning(&source, Value, "held", 1).expect("the call names the declaration");
    assert_eq!(called.kind, DefinitionKind::Function);
    assert_eq!(meaning(&source, Value, "held", 2), Some(called));
}

#[test]
fn a_name_after_the_dot_that_nothing_declares_is_refused_where_it_is_written() {
    let source = calling_held("    count := 1\n    count.missing(2)\n");

    assert_eq!(
        refusal(&source).message(),
        "there is nothing named `missing`"
    );
}

#[test]
fn the_receiver_is_resolved_as_the_value_it_is() {
    let source = calling_held("    missing.held(2)\n");

    assert_eq!(
        refusal(&source).message(),
        "there is nothing named `missing`"
    );
}

#[test]
fn anything_at_all_stands_before_the_dot_where_it_is_a_value() {
    let source = calling_held("    held(1, 2).held(3)\n");

    resolved(&source);
}

#[test]
fn a_field_read_leaves_the_name_after_the_dot_to_the_type_it_is_read_through() {
    let source = "fn named(user: User) -> String {\n    user.name\n}\n\ntype User = {\n    name: String\n}\n";

    assert_eq!(meaning(source, Value, "name", 1), None);
}

#[test]
fn a_function_reached_through_a_module_is_no_name_of_this_one() {
    let source = "import io\n\nfn go() -> () {\n    io.println(\"hi\")\n}\n";

    assert_eq!(meaning(source, Value, "println", 1), None);
}
