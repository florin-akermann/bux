//! The functions of a library module the compiler holds, rather than that module's own source.
//!
//! `docs/specs/library.md` states which they are and why. `List` is a type the JVM holds, so what
//! a list does that no Bux body says is the compiler's too: `push` builds a list from a list,
//! which no expression the grammar writes does, and `at` reads one at an index in the time the
//! JVM reads one slot in. Neither is an `extern` either, because an `extern` signature writes no
//! type parameter and each of these writes one.

use crate::scheme::{Quantified, Scheme};
use crate::surface::Offered;
use crate::types::{Type, TypeParameter};

/// The one module the compiler holds a function of.
const LIST: &str = "list";

/// The name of the one that grows a list.
const PUSH: &str = "push";

/// The name of the one that reads a list at an index.
const AT: &str = "at";

/// What the element of a list is called where a signature here writes it.
const ELEMENT: &str = "T";

/// What the compiler holds for `module`, which is nothing at all for every module but one.
pub(crate) fn offered_by_the_compiler(module: &str) -> Vec<(String, Offered)> {
    if module != LIST {
        return Vec::new();
    }
    vec![
        (PUSH.to_owned(), Offered::of(push())),
        (AT.to_owned(), Offered::of(at())),
    ]
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
