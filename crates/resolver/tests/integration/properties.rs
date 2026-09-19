//! The invariants of `docs/specs/modules.md`, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_parser::parse;
use lumen_resolver::resolve;

/// The pieces a generated module is built from, each resolving on its own and declaring its own
/// names, so any set of them is a module that resolves.
const PIECES: [&str; 6] = [
    "import io",
    "type UserId = UserId(Int64)",
    "type User = {\n    id: Int64\n}",
    "fn identity<T>(value: T) -> T {\n    value\n}",
    "fn total(counts: List<Int>) -> Int {\n    var sum = 0\n    for count in counts {\n        sum += 1\n    }\n    sum\n}",
    "type Payment =\n    | Pending\n    | Failed(String)\n\nfn describe(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}",
];

#[hegel::test]
fn resolving_never_panics_and_is_deterministic(tc: TestCase) {
    let source = tc.draw(gs::text());
    let Ok(program) = parse(&source) else {
        return;
    };
    assert_eq!(resolve(program.clone()), resolve(program));
}

#[hegel::test]
fn a_module_built_of_pieces_that_each_resolve_resolves(tc: TestCase) {
    let source = module(&tc);
    let program = parse(&source).expect("a well-formed module parses");
    resolve(program).unwrap_or_else(|error| panic!("{source:?} resolves: {}", error.message()));
}

/// A module of distinct [`PIECES`]; a piece drawn twice would declare its names twice.
fn module(tc: &TestCase) -> String {
    let mut chosen: Vec<&'static str> = Vec::new();
    for piece in tc.draw(gs::vecs(gs::sampled_from(&PIECES))) {
        if !chosen.contains(&piece) {
            chosen.push(piece);
        }
    }
    chosen.join("\n\n")
}

#[hegel::test]
fn a_refusal_points_inside_the_source(tc: TestCase) {
    let source = tc.draw(gs::text());
    let Ok(program) = parse(&source) else {
        return;
    };
    let Err(error) = resolve(program) else {
        return;
    };
    let span = error.span();
    assert!(span.start() < span.end(), "{span:?} is empty");
    assert!(span.end() <= source.len(), "{span:?} runs past the input");
}
