//! What a refusal about an `extern` declaration says, and the rule it says it against.
//!
//! `docs/specs/interop.md` states these, and they read as one group: each says that the boundary
//! is a signature and the Java name beside it and nothing more. A kind reaches either function
//! only from the arm of its caller that names it, which is why the last arm of each panics.

use super::TypeErrorKind;

/// The rule for a refusal about an `extern` declaration and what it reaches.
pub(super) const fn the_rule(kind: &TypeErrorKind) -> &'static str {
    match kind {
        TypeErrorKind::DoesNotCross { .. } => {
            "a boundary carries `Bool`, `Int`, `String`, and a type an `extern` names"
        }
        TypeErrorKind::NotAJavaName(_) => {
            "a Java name is its segments, each a name, with a dot between two of them"
        }
        TypeErrorKind::DerivesAForeignType(_) => {
            "write an `instance` over `extern` declarations instead"
        }
        TypeErrorKind::ReachesNoClass(_) => {
            "a class is `String`, or a type an `extern type` declares"
        }
        TypeErrorKind::BuildsAnInterface(_) => {
            "a `new` builds a class; reach one through a member of a class instead"
        }
        TypeErrorKind::WidensNoInt(_) => {
            "an `int` widens to an `Int`; drop the word, or give an `Int` back"
        }
        _ => panic!("a kind reaches here only from the arm of `stated` that names it"),
    }
}

/// What the refusal could not reach, in the words the reader is shown.
pub(super) fn what_it_reached(kind: &TypeErrorKind) -> String {
    match kind {
        TypeErrorKind::DoesNotCross { written, crossing } => {
            let place = crossing.what_a_member_does();
            format!("`{written}` is no type a Java member {place}")
        }
        TypeErrorKind::NotAJavaName(written) => format!("`{written}` is no Java name"),
        TypeErrorKind::DerivesAForeignType(named) => {
            format!("`{named}` is an extern type, and a derive reads what a type holds")
        }
        TypeErrorKind::ReachesNoClass(reaches) => {
            format!("a `{reaches}` reaches a class, and this signature names none")
        }
        TypeErrorKind::BuildsAnInterface(built) => {
            format!("a `new` builds a class, and `{built}` is an interface")
        }
        TypeErrorKind::WidensNoInt(written) => {
            format!("`int` widens to an `Int`, and this signature gives back `{written}`")
        }
        _ => panic!("a kind reaches here only from the arm of `Display` that names it"),
    }
}
