//! Helpers shared by the hole behaviour tests.
//!
//! A behaviour is stated as a whole module, because that is what the walk takes: a hole is a
//! name meaning what the prelude says it means, which only a resolved module settles.

use lumen_holes::{Hole, Whole};
use lumen_parser::parse;
use lumen_resolver::resolve;
use lumen_types::TypedProgram;

/// What each hole of `source` is written as, which is the text its span covers.
pub fn written(source: &str) -> Vec<&str> {
    holes(source)
        .iter()
        .map(|hole| hole.span().text(source))
        .collect()
}

/// Every hole of `source`, in the order they are written.
pub fn holes(source: &str) -> Vec<Hole> {
    Whole::of_module(&typed(source)).err().unwrap_or_default()
}

/// The inferred program of `source`, which must get that far.
pub fn typed(source: &str) -> TypedProgram {
    let program = parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"));
    let resolved =
        resolve(program).unwrap_or_else(|error| panic!("{source:?} resolves: {error:?}"));
    lumen_types::check(resolved, &lumen_types::Imported::default())
        .unwrap_or_else(|error| panic!("{source:?} is typed: {error:?}"))
}
