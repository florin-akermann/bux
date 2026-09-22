//! Helpers shared by the resolver's behaviour tests.
//!
//! A behaviour is stated as a whole module, because that is what resolution takes. What one name
//! of it means is then read back by the text it is written with.

use lumen_ast::{Name, Span};
use lumen_parser::parse;
use lumen_resolver::{Definition, Namespace, ResolveError, ResolvedProgram, resolve};

/// The name the module under test is compiled as, which nothing here depends on.
const MODULE: &str = "demo";

/// The failure `source` is refused with.
pub fn refusal(source: &str) -> ResolveError {
    let program = parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"));
    resolve(program, MODULE)
        .err()
        .unwrap_or_else(|| panic!("{source:?} is refused"))
}

/// What the occurrence of `written` numbered `occurrence` means, counting from one.
pub fn meaning(
    source: &str,
    namespace: Namespace,
    written: &str,
    occurrence: usize,
) -> Option<Definition> {
    let at = source
        .match_indices(written)
        .nth(occurrence - 1)
        .unwrap_or_else(|| panic!("{source:?} writes {written:?} {occurrence} times"))
        .0;
    let name = Name {
        text: written.to_owned(),
        span: Span::new(at, written.len()),
    };
    resolved(source).definition(namespace, &name)
}

/// The resolved program of `source`, which must resolve.
pub fn resolved(source: &str) -> ResolvedProgram {
    let program = parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"));
    resolve(program, MODULE)
        .unwrap_or_else(|error| panic!("{source:?} resolves: {}", error.message()))
}
