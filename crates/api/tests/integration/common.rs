//! Helpers shared by the API-surface behaviour tests.
//!
//! A behaviour is stated as a whole module, because a page is about a module: what a name is
//! reached as, and what type it has, are both settled by the module it is written in.

use lumen_ast::Program;
use lumen_parser::parse;
use lumen_resolver::resolve;

/// The name the module under test is compiled as, which nothing here depends on.
const MODULE: &str = "demo";

/// The page `source` has, which `docs/specs/api-surface.md` states.
pub fn page(source: &str) -> String {
    let resolved = resolve(tree(source), MODULE)
        .unwrap_or_else(|error| panic!("{source:?} resolves: {error:?}"));
    let typed = lumen_types::check(resolved, &lumen_types::Imported::default())
        .unwrap_or_else(|error| panic!("{source:?} is typed: {error:?}"));
    lumen_api::surface(&typed)
}

/// The tree `source` parses to, which is where the names it declares are read from.
pub fn tree(source: &str) -> Program {
    parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"))
}
