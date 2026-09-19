//! The invariants of `docs/specs/types.md`, checked on generated input.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_parser::parse;
use lumen_resolver::resolve;
use lumen_types::check;

use crate::common::{expressions, inferred_type};

/// The pieces a generated module is built from, each inferring on its own and declaring its own
/// names, so any set of them is a module that infers.
const PIECES: [&str; 6] = [
    "type UserId = UserId(Int)",
    "fn open(user: User) -> Bool {\n    user.active\n}\n\ntype User = {\n    id: Int\n    active: Bool\n}\n",
    "fn identity<T>(value: T) -> T {\n    value\n}",
    "fn total(counts: List<Int>) -> Int {\n    var sum = 0\n    for count in counts {\n        sum += count\n    }\n    sum\n}",
    "fn shout(word: String) -> String {\n    word + \"!\"\n}",
    "fn describe(payment: Payment) -> String {\n    match payment {\n        Pending => \"waiting\"\n        Failed(reason) => reason\n    }\n}\n\ntype Payment =\n    | Pending\n    | Failed(String)\n",
];

/// The pieces that are refused however sound the module around them is.
const REFUSALS: [&str; 4] = [
    "fn refused() -> Int {\n    \"seven\"\n}",
    "fn refused<T>(value: T) -> T {\n    1\n}",
    "fn refused(flag: Bool) -> Bool {\n    flag + flag\n}",
    "fn refused(value: Int) -> Int {\n    value.missing\n}",
];

#[hegel::test]
fn inferring_never_panics_and_is_deterministic(tc: TestCase) {
    let source = tc.draw(gs::text());
    let Ok(program) = parse(&source) else {
        return;
    };
    let Ok(resolved) = resolve(program) else {
        return;
    };
    assert_eq!(check(resolved.clone()).is_ok(), check(resolved).is_ok());
}

#[hegel::test]
fn a_module_built_of_pieces_that_each_infer_infers(tc: TestCase) {
    let source = module(&tc);
    let resolved = resolved(&source);
    check(resolved).unwrap_or_else(|error| panic!("{source:?} infers: {}", error.message()));
}

#[hegel::test]
fn a_refusal_points_inside_the_source(tc: TestCase) {
    let refused = tc.draw(gs::sampled_from(&REFUSALS));
    let source = format!("{}\n\n{refused}\n", module(&tc));
    let error = check(resolved(&source))
        .err()
        .unwrap_or_else(|| panic!("{source:?} is refused"));
    let span = error.span();

    assert!(span.start() < span.end(), "{span:?} is empty");
    assert!(span.end() <= source.len(), "{span:?} runs past the input");
}

#[hegel::test]
fn every_expression_of_an_inferred_program_has_a_type(tc: TestCase) {
    let source = module(&tc);
    let typed = check(resolved(&source)).expect("a well-formed module infers");
    for expr in expressions(typed.resolved().program()) {
        assert!(
            typed.type_of(expr.span).is_some(),
            "{:?} has no type",
            expr.span.text(&source)
        );
    }
}

#[hegel::test]
fn no_expression_of_an_inferred_module_is_left_unsettled(tc: TestCase) {
    let source = module(&tc);
    let typed = check(resolved(&source)).expect("a well-formed module infers");
    for expr in expressions(typed.resolved().program()) {
        let inferred = typed.type_of(expr.span).expect("an expression has a type");
        assert!(
            !inferred.to_string().contains('_'),
            "{:?} was left unsettled as {inferred}",
            expr.span.text(&source)
        );
    }
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

fn resolved(source: &str) -> lumen_resolver::ResolvedProgram {
    let program = parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"));
    resolve(program).unwrap_or_else(|error| panic!("{source:?} resolves: {}", error.message()))
}

/// A value of each type a generic function can be used at, with the type it has.
const VALUES: [(&str, &str); 3] = [("1", "Int"), ("\"two\"", "String"), ("true", "Bool")];

/// A generic function is inferred to its most general type, so no pair of uses is too much for it.
#[hegel::test]
fn a_generic_function_is_general_enough_for_any_two_uses(tc: TestCase) {
    let (first, first_type) = tc.draw(gs::sampled_from(&VALUES));
    let (second, second_type) = tc.draw(gs::sampled_from(&VALUES));
    let source = format!(
        "fn go() -> {second_type} {{\n    one := given({first})\n    two := given({second})\n    two\n}}\n\n\
         fn given<T>(value: T) -> T {{\n    value\n}}\n"
    );

    assert_eq!(
        inferred_type(&source, &format!("given({first})"), 1),
        first_type
    );
    assert_eq!(
        inferred_type(&source, &format!("given({second})"), 1),
        second_type
    );
}
