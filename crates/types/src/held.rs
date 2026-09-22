//! The functions of a library module the compiler holds, rather than that module's own source.
//!
//! `docs/specs/library.md` states which they are and why. `List` is a type the JVM holds, so what
//! a list does that no Bux body says is the compiler's too: `push` builds a list from a list,
//! which no expression the grammar writes does, and `at` reads one at an index in the time the
//! JVM reads one slot in. Neither is an `extern` either, because an `extern` signature writes no
//! type parameter and each of these writes one.

use lumen_resolver::prelude;

use crate::scheme::{Quantified, Scheme};
use crate::surface::Offered;
use crate::types::{Type, TypeParameter};

/// What the element of a list is called where a signature here writes it.
const ELEMENT: &str = "T";

/// What `module` offers of what the compiler holds, which is nothing for every module but one.
pub(crate) fn offered_by_the_compiler(module: &str) -> Vec<(String, Offered)> {
    schemes_of(prelude::offered_by(module))
        .into_iter()
        .map(|(name, scheme)| (name.to_owned(), Offered::of(scheme)))
        .collect()
}

/// What the source of `module` writes of them, which `docs/specs/library.md` says is the
/// prelude's alone: it imports nothing, and its instances over `List<T>` read a list with `at`.
pub(crate) fn held_for(module: &str) -> Vec<(&'static str, Scheme)> {
    schemes_of(prelude::held_for(module))
}

/// Each of those names with the type the compiler gives it.
fn schemes_of(names: Vec<&'static str>) -> Vec<(&'static str, Scheme)> {
    names
        .into_iter()
        .map(|name| (name, written_as(name)))
        .collect()
}

/// The type the function the compiler holds under `name` has.
fn written_as(name: &str) -> Scheme {
    if name == prelude::PUSH {
        return push();
    }
    at()
}

/// `push<T>(values: List<T>, value: T) -> List<T>`.
fn push() -> Scheme {
    over(&|element: Type| {
        Type::function(
            vec![Type::list(element.clone()), element.clone()],
            Type::list(element),
        )
    })
}

/// `at<T>(values: List<T>, index: Int) -> Option<T>`.
fn at() -> Scheme {
    over(&|element: Type| {
        Type::function(
            vec![Type::list(element.clone()), Type::int()],
            Type::option(element),
        )
    })
}

/// A scheme over the one type parameter each of these writes, which is what a list holds.
fn over(written: &dyn Fn(Type) -> Type) -> Scheme {
    let element = TypeParameter::prelude(ELEMENT);
    Scheme::over(
        vec![Quantified::Parameter(element.clone())],
        written(Type::Parameter(element)),
    )
}
