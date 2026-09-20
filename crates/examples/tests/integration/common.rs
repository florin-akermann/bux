//! Helpers shared by the example behaviour tests.
//!
//! A behaviour is stated as a whole module, because an example is read out of the comment above
//! a declaration: the text and the tree are both needed, and a fragment of either is neither.

use lumen_ast::Program;
use lumen_diagnostics::render;
use lumen_examples::{Example, Refusal, Run, stated_by};
use lumen_parser::parse;

/// What each example `source` states is written as, in the order they are written.
pub fn expressions(source: &str) -> Vec<String> {
    stated(source)
        .iter()
        .map(|example| example.expression().to_owned())
        .collect()
}

/// The module a run of `source` writes, which is what `lumen test` compiles.
pub fn run(source: &str) -> Run {
    Run::of_module(source, &parsed(source), stated(source))
        .expect("the module states examples and declares no name the run reaches")
}

/// The examples `source` states, which it must state without being refused.
pub fn stated(source: &str) -> Vec<Example> {
    stated_by(source, &parsed(source))
        .unwrap_or_else(|refused| panic!("{source:?} states its examples: {refused:?}"))
}

/// Each refusal of `source` as the page a reader sees, message and rendered line included.
pub fn pages(source: &str) -> Vec<String> {
    refusals(source)
        .iter()
        .map(|refusal| render(&refusal.diagnostic(), source, "demo.lm"))
        .collect()
}

/// Every refusal `source` earns, in the order they are written.
pub fn refusals(source: &str) -> Vec<Refusal> {
    stated_by(source, &parsed(source)).err().unwrap_or_default()
}

/// The tree of `source`, which must parse for any of this to mean anything.
pub fn parsed(source: &str) -> Program {
    parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"))
}
