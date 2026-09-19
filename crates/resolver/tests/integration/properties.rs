//! The invariants of `docs/specs/modules.md`, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ast::{Item, Name, Program};
use lumen_parser::parse;
use lumen_resolver::{Namespace, Origin, resolve};

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

/// The pieces that refuse however sound the module around them is.
const REFUSALS: [&str; 4] = [
    "fn refused() -> Int {\n    nothing_names_this\n}",
    "fn refused() -> Missing {\n    1\n}",
    "fn refused(seen: Int) -> Int {\n    seen := 1\n    seen\n}",
    "fn Ok() -> Int {\n    1\n}",
];

#[hegel::test]
fn a_refusal_points_inside_the_source(tc: TestCase) {
    let refused = tc.draw(gs::sampled_from(&REFUSALS));
    let source = format!("{}\n\n{refused}\n", module(&tc));
    let program = parse(&source).expect("a well-formed module parses");
    let error = resolve(program)
        .err()
        .unwrap_or_else(|| panic!("{source:?} is refused"));
    let span = error.span();

    assert!(span.start() < span.end(), "{span:?} is empty");
    assert!(span.end() <= source.len(), "{span:?} runs past the input");
}

#[hegel::test]
fn every_name_a_module_declares_is_declared_at_that_name(tc: TestCase) {
    let source = module(&tc);
    let program = parse(&source).expect("a well-formed module parses");
    let resolved = resolve(program).expect("a well-formed module resolves");
    for (namespace, name) in declared_names(resolved.program()) {
        let definition = resolved
            .definition(namespace, name)
            .unwrap_or_else(|| panic!("{} names something", name.text));
        assert_eq!(definition.origin, Origin::Declared(name.span));
        assert_eq!(name.span.text(&source), name.text);
    }
}

/// Every name a module writes as a declaration, with the namespace it declares into.
fn declared_names(program: &Program) -> Vec<(Namespace, &Name)> {
    let mut found = Vec::new();
    for item in &program.items {
        match item {
            Item::Import(import) => found.push((Namespace::Value, &import.module)),
            Item::Function(function) => found.push((Namespace::Value, &function.name)),
            Item::Type(declaration) => found.push((Namespace::Type, &declaration.name)),
        }
    }
    found
}
