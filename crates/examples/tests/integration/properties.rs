//! The properties of `docs/specs/doc-examples.md`, over modules drawn rather than written out.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ast::Span;

use crate::common::{expressions, refusals, run, stated};

/// The names a drawn module declares its functions under.
const NAMED: [&str; 5] = ["held", "shared", "counted", "named", "carried"];

/// How many functions a drawn module declares.
const MANY: [usize; 5] = [1, 2, 3, 4, 5];

/// A module of `many` functions, each stating one example of itself.
fn documented(many: usize) -> String {
    let declared = NAMED.iter().take(many).map(|named| {
        format!(
            "// example: {named}(total: 7) == 7\nfn {named}(total: Int) -> Int {{\n    total\n}}\n"
        )
    });
    declared.collect::<Vec<String>>().join("\n")
}

#[hegel::test]
fn every_example_a_module_states_is_found_however_many_functions_it_declares(tc: TestCase) {
    let many = tc.draw(gs::sampled_from(&MANY));

    assert_eq!(expressions(&documented(many)).len(), many);
}

#[hegel::test]
fn a_module_whose_every_function_states_an_example_is_accepted(tc: TestCase) {
    let many = tc.draw(gs::sampled_from(&MANY));

    assert_eq!(refusals(&documented(many)).len(), 0);
}

#[hegel::test]
fn the_text_of_an_example_is_the_text_of_its_line_after_the_marker(tc: TestCase) {
    let many = tc.draw(gs::sampled_from(&MANY));
    let source = documented(many);

    for example in stated(&source) {
        let line = example.span().text(&source);
        assert_eq!(line, format!("// example: {}", example.expression()));
    }
}

#[hegel::test]
fn a_span_a_run_reports_is_a_span_of_the_file_the_examples_were_read_out_of(tc: TestCase) {
    let many = tc.draw(gs::sampled_from(&MANY));
    let source = documented(many);
    let written = run(&source);
    let offsets: Vec<usize> = (0..=written.source().len()).collect();

    let points = written.in_original(Span::new(tc.draw(gs::sampled_from(&offsets)), 1));

    assert!(
        points.end() <= source.len(),
        "{points:?} is outside {source:?}"
    );
}
