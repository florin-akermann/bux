//! Helpers shared by the exhaustiveness behaviour tests.
//!
//! A behaviour is stated as a whole module, because that is what the check takes. A module either
//! passes the check or names the values one of its matches leaves uncovered.

use lumen_exhaustiveness::{MatchError, check};
use lumen_parser::parse;
use lumen_resolver::resolve;
use lumen_types::TypedProgram;

/// The failure `source` is refused with.
pub fn refusal(source: &str) -> MatchError {
    check(&typed(source))
        .err()
        .unwrap_or_else(|| panic!("{source:?} is refused"))
}

/// Asserts that every `match` of `source` covers the type it matches.
pub fn covers_everything(source: &str) {
    if let Err(error) = check(&typed(source)) {
        panic!("{source:?} covers everything: {}", error.message());
    }
}

/// The inferred program of `source`, which must get that far.
pub fn typed(source: &str) -> TypedProgram {
    let program = parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"));
    let resolved =
        resolve(program).unwrap_or_else(|error| panic!("{source:?} resolves: {error:?}"));
    lumen_types::check(resolved, &lumen_types::Imported::default())
        .unwrap_or_else(|error| panic!("{source:?} infers: {}", error.message()))
}

/// A module matching on a nested value, with `arms` as the arms of the one `match`.
///
/// `Option<Result<Int, String>>` is the smallest type whose values name two declarations, so an
/// arm of it is placed by the variant it answers for and then by what it reaches for.
pub fn outcomes(arms: &str) -> String {
    format!(
        concat!(
            "fn described(outcome: Option<Result<Int, String>>) -> String {{\n",
            "    match outcome {{\n{arms}    }}\n}}\n"
        ),
        arms = arms
    )
}

/// A module declaring the payment type of `docs/design.md`, with `body` as the one function.
pub fn payments(body: &str) -> String {
    format!(
        concat!(
            "fn describe(payment: Payment) -> String {{\n{body}\n}}\n\n",
            "type Payment =\n    | Pending\n    | Authorized {{\n        authorization_id: String\n    }}\n    | Failed(String)\n"
        ),
        body = body
    )
}
