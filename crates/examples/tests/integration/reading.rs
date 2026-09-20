//! The behaviours of `docs/specs/doc-examples.md`: what a module states, and what it fails to.

use crate::common::{expressions, pages, refusals, stated};

/// A module whose one function states `written` above it, and nothing else.
fn stating(written: &str) -> String {
    format!("{written}fn shared(total: Int) -> Int {{\n    total\n}}\n")
}

#[test]
fn a_function_states_the_example_written_above_it() {
    let source = stating("// example: shared(total: 7) == 7\n");

    assert_eq!(expressions(&source), vec!["shared(total: 7) == 7"]);
}

#[test]
fn an_example_is_the_text_written_after_the_marker() {
    let source = stating("//    example:   shared(total: 7) == 7   \n");

    assert_eq!(expressions(&source), vec!["shared(total: 7) == 7"]);
}

#[test]
fn a_doc_comment_states_an_example_as_a_plain_comment_does() {
    let source = stating("/// example: shared(total: 7) == 7\n");

    assert_eq!(expressions(&source), vec!["shared(total: 7) == 7"]);
}

#[test]
fn a_function_states_every_example_written_above_it_in_the_order_they_are_written() {
    let source = stating(concat!(
        "// example: shared(total: 7) == 7\n",
        "// example: shared(total: 8) == 8\n",
    ));

    assert_eq!(
        expressions(&source),
        vec!["shared(total: 7) == 7", "shared(total: 8) == 8"]
    );
}

#[test]
fn a_comment_that_is_not_an_example_is_not_read_as_one() {
    let source = stating(concat!(
        "// Gives back whatever it was handed, which is as much as it does.\n",
        "//\n",
        "// example: shared(total: 7) == 7\n",
    ));

    assert_eq!(expressions(&source), vec!["shared(total: 7) == 7"]);
}

#[test]
fn an_example_names_the_function_it_is_written_above() {
    let source = stating("// example: shared(total: 7) == 7\n");

    assert_eq!(stated(&source)[0].documents(), "shared");
}

#[test]
fn an_example_points_at_the_whole_line_it_is_written_on() {
    let source = stating("// example: shared(total: 7) == 7\n");

    assert_eq!(
        stated(&source)[0].span().text(&source),
        "// example: shared(total: 7) == 7"
    );
}

#[test]
fn a_blank_line_between_the_comment_and_the_function_ends_the_comment() {
    let source = stating("// example: shared(total: 7) == 7\n\n");

    assert_eq!(refusals(&source).len(), 2);
}

#[test]
fn a_function_that_states_none_is_refused_where_it_is_declared() {
    let source = stating("");

    let page = pages(&source).join("");
    assert!(
        page.contains("error[L0601]: `shared` states no example"),
        "{page}"
    );
    assert!(page.contains("fn shared(total: Int) -> Int {"), "{page}");
}

#[test]
fn every_function_that_states_none_is_named_and_not_only_the_first() {
    let source = concat!(
        "fn one(total: Int) -> Int {\n    total\n}\n\n",
        "fn two(total: Int) -> Int {\n    total\n}\n",
    );

    let page = pages(source).join("");
    assert!(page.contains("`one` states no example"), "{page}");
    assert!(page.contains("`two` states no example"), "{page}");
}

#[test]
fn the_main_a_module_declares_is_reached_by_running_it_and_states_none() {
    let source = "fn main() -> () {\n    7\n}\n";

    assert_eq!(refusals(source).len(), 0);
}

#[test]
fn an_example_above_main_documents_nothing() {
    let source = "// example: 1 == 1\nfn main() -> () {\n    7\n}\n";

    let page = pages(source).join("");
    assert!(
        page.contains("error[L0602]: this example documents nothing"),
        "{page}"
    );
}

#[test]
fn an_example_above_a_type_documents_nothing() {
    let source = concat!(
        "fn main() -> () {\n    7\n}\n\n",
        "// example: 1 == 1\ntype User = {\n    id: Int\n}\n",
    );

    let page = pages(source).join("");
    assert!(page.contains("error[L0602]"), "{page}");
}

#[test]
fn an_example_written_inside_a_body_documents_nothing() {
    let source = "fn main() -> () {\n    // example: 1 == 1\n    7\n}\n";

    let page = pages(source).join("");
    assert!(page.contains("error[L0602]"), "{page}");
}

#[test]
fn an_example_at_the_end_of_a_line_of_code_documents_nothing() {
    let source = concat!(
        "fn main() -> () {\n    7\n} // example: shared(total: 7) == 7\n",
        "fn shared(total: Int) -> Int {\n    total\n}\n",
    );

    let page = pages(source).join("");
    assert!(page.contains("error[L0602]"), "{page}");
    assert!(page.contains("`shared` states no example"), "{page}");
}

#[test]
fn a_refusal_is_reported_in_the_order_it_is_written() {
    let source = concat!(
        "// example: 1 == 1\nfn main() -> () {\n    7\n}\n\n",
        "fn shared(total: Int) -> Int {\n    total\n}\n",
    );

    let written = pages(source);
    assert!(written[0].contains("L0602"), "{written:?}");
    assert!(written[1].contains("L0601"), "{written:?}");
}
