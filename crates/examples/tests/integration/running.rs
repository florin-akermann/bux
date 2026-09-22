//! The behaviours of `docs/specs/doc-examples.md`: the module a run writes, and what it points at.

use lumen_ast::Span;
use lumen_examples::Run;

use crate::common::{parsed, run, stated};

/// A module that states one example of `shared` and runs nothing of its own.
const SHARED: &str = concat!(
    "// example: shared(total: 7) == 7\n",
    "fn shared(total: Int) -> Int {\n    total\n}\n",
);

#[test]
fn a_written_module_opens_with_the_import_the_run_writes_with() {
    let written = run(SHARED);

    assert!(written.source().starts_with("import io\n\n"), "{written:?}");
}

#[test]
fn a_module_that_imports_already_keeps_its_imports_sorted_and_named_once() {
    let source = format!("import files\n\nimport io\n\n{SHARED}");

    let written = run(&source);

    assert!(
        written
            .source()
            .starts_with("import files\n\nimport io\n\nfn main"),
        "{written:?}"
    );
}

#[test]
fn a_written_module_tries_each_example_and_writes_the_one_that_did_not_hold() {
    let written = run(SHARED);

    assert!(
        written.source().contains(concat!(
            "fn main(arguments: List<String>) -> Int {\n",
            "    if !(shared(total: 7) == 7) {\n",
            "        io.println(\"lumen: example 0\")\n",
            "    }\n",
            "    0\n",
            "}\n",
        )),
        "{written:?}"
    );
}

#[test]
fn a_written_module_carries_the_declarations_the_module_makes_comment_and_all() {
    let written = run(SHARED);

    assert!(written.source().ends_with(SHARED), "{written:?}");
}

#[test]
fn a_written_module_leaves_out_the_main_the_module_declares_of_its_own() {
    let source = format!("fn main(arguments: List<String>) -> Int {{\n    7\n}}\n\n{SHARED}");

    let written = run(&source);

    assert_eq!(written.source().matches("fn main").count(), 1);
}

#[test]
fn a_line_the_run_writes_names_the_example_it_is_about() {
    let written = run(SHARED);

    let line = format!("{}0", Run::marks());

    let named = written.named(&line).expect("the run writes one example");
    assert_eq!(named.documents(), "shared");
}

#[test]
fn a_line_that_names_no_example_names_nothing() {
    let written = run(SHARED);

    assert!(written.named("the file was read").is_none());
}

#[test]
fn a_line_the_program_wrote_itself_names_no_example_however_it_reads() {
    let written = run(SHARED);

    assert!(written.named("0").is_none());
}

#[test]
fn a_module_that_declares_the_name_the_run_reaches_for_is_refused() {
    let source = format!(
        "{SHARED}\n// example: io(total: 7) == 7\nfn io(total: Int) -> Int {{\n    total\n}}\n"
    );
    let program = parsed(&source);

    let refused = Run::of_module(&source, &program, stated(&source))
        .expect_err("the module declares `io`, which the run reaches for");

    assert_eq!(refused.span().text(&source), "io");
}

#[test]
fn a_written_module_starts_at_the_one_shape_a_program_starts_at() {
    let written = run(SHARED);

    assert!(
        written
            .source()
            .contains("fn main(arguments: List<String>) -> Int {\n"),
        "{written:?}"
    );
}

#[test]
fn a_module_that_declares_the_name_the_written_entry_takes_is_refused() {
    let source =
        format!("{SHARED}\n// example: arguments() == 7\nfn arguments() -> Int {{\n    7\n}}\n");
    let program = parsed(&source);

    let refused = Run::of_module(&source, &program, stated(&source))
        .expect_err("the module declares `arguments`, which the `main` the run writes takes");

    assert_eq!(refused.span().text(&source), "arguments");
}

#[test]
fn a_span_inside_an_example_points_at_the_line_the_example_is_written_on() {
    let written = run(SHARED);
    let at = written
        .source()
        .find("shared(total: 7) == 7")
        .expect("the run writes the example it was given");

    let points = written.in_original(Span::new(at, "shared".len()));

    assert_eq!(points.text(SHARED), "// example: shared(total: 7) == 7");
}

#[test]
fn a_span_inside_copied_source_points_at_the_same_text_in_the_file_it_came_from() {
    let written = run(SHARED);
    let at = written
        .source()
        .find("total: Int")
        .expect("the run copies the declaration it was given");

    let points = written.in_original(Span::new(at, "total".len()));

    assert_eq!(points.text(SHARED), "total");
}

#[test]
fn a_span_in_what_the_run_wrote_for_itself_points_at_the_module_as_a_whole() {
    let written = run(SHARED);

    let points = written.in_original(Span::new(0, "import".len()));

    assert_eq!(points.text(SHARED), SHARED);
}
