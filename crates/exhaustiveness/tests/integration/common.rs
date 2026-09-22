//! Helpers shared by the exhaustiveness behaviour tests.
//!
//! A behaviour is stated as a whole module, because that is what the check takes. A module either
//! passes the check or names the values one of its matches leaves uncovered.

use lumen_exhaustiveness::{MatchError, check};
use lumen_parser::parse;
use lumen_resolver::resolve;
use lumen_types::{Imported, TypedProgram};

/// The name the module under test is compiled as, which nothing here depends on.
const MODULE: &str = "demo";

/// The failure `source` is refused with.
pub fn refusal(source: &str) -> MatchError {
    reaching(source).refusal()
}

/// Asserts that every `match` of `source` covers the type it matches.
pub fn covers_everything(source: &str) {
    reaching(source).covers_everything();
}

/// A module importing `demo` and matching one of its payments, with `arms` as the arms.
///
/// `demo` declares the payment type of `docs/design.md`, so every arm names a variant another
/// module declares and the check reads them through what that module offers.
pub fn demo_payments(arms: &str) -> Reaching {
    let demo = concat!(
        "fn pending() -> Payment {\n    Pending\n}\n\n",
        "type Payment =\n    | Pending\n    | Authorized {\n        authorization_id: String\n    }\n    | Failed(String)\n"
    );
    Reaching {
        source: format!(
            "import demo\n\nfn describe(payment: demo.Payment) -> String {{\n    match payment {{\n{arms}    }}\n}}\n"
        ),
        imported: Imported::default().offering("demo", typed(demo).surface().clone()),
    }
}

/// The inferred program of `source`, which must get that far.
pub fn typed(source: &str) -> TypedProgram {
    reaching(source).typed()
}

/// A module under test, with what the modules it imports offer it.
pub struct Reaching {
    source: String,
    imported: Imported,
}

impl Reaching {
    /// The failure the module is refused with.
    pub fn refusal(&self) -> MatchError {
        check(&self.typed())
            .err()
            .unwrap_or_else(|| panic!("{:?} is refused", self.source))
    }

    /// Asserts that every `match` of the module covers the type it matches.
    pub fn covers_everything(&self) {
        if let Err(error) = check(&self.typed()) {
            panic!("{:?} covers everything: {}", self.source, error.message());
        }
    }

    /// The inferred program of the module, which must get that far.
    pub fn typed(&self) -> TypedProgram {
        let source = &self.source;
        let program = parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"));
        let resolved = resolve(program, MODULE)
            .unwrap_or_else(|error| panic!("{source:?} resolves: {error:?}"));
        lumen_types::check(resolved, &self.imported)
            .unwrap_or_else(|error| panic!("{source:?} infers: {}", error.message()))
    }
}

/// `source` as a module under test that imports nothing.
fn reaching(source: &str) -> Reaching {
    Reaching {
        source: source.to_owned(),
        imported: Imported::default(),
    }
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
