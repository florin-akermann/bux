//! The order a `match` lists its arms in, which is the order the type declares its variants.
//!
//! `docs/design.md` section 13 asks for it so that a new variant has exactly one place to be
//! handled and no diff is ever reorder-only. The rule is checked here because this is the phase
//! that knows the variant list; the order is said, never rewritten.
//!
//! An arm is placed by the constructors it writes, read left to right and outermost first, so
//! two arms that reach inside one variant are placed by what they reach for. Placing stops at
//! the first thing no declaration writes down: a name that binds, a number, a string, and the
//! two values of a `Bool` are each written where the author put them, and so is everything an
//! arm writes after one of them.

use lumen_ast::{MatchExpr, Path, Pattern, PatternKind, Span};

use crate::pattern::Reading;
use crate::space::Space;

/// The first arm of `matching` written above one the declarations put it below, when one is.
pub(crate) fn out_of_order(
    matching: &MatchExpr,
    reading: &Reading,
    space: &Space,
) -> Option<OutOfOrder> {
    let placed: Vec<Vec<Step>> = matching
        .arms
        .iter()
        .map(|arm| steps(&arm.pattern, reading, space))
        .collect();
    for (at, written) in placed.iter().enumerate() {
        let span = matching.arms[at].pattern.span;
        let found = placed[..at]
            .iter()
            .find_map(|above| out_of_place(written, above, span));
        if found.is_some() {
            return found;
        }
    }
    None
}

/// An arm written above one the type declares above it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OutOfOrder {
    /// The constructor that puts the arm lower, which is the one the reader is pointed at.
    arm: String,
    /// The constructor of an arm above it, which its type declares below `arm`.
    before: String,
    span: Span,
}

impl OutOfOrder {
    pub(crate) const fn span(&self) -> Span {
        self.span
    }

    pub(crate) fn message(&self) -> String {
        format!("this `match` writes `{}` after `{}`", self.arm, self.before)
    }

    pub(crate) fn help(&self) -> String {
        format!(
            "arms come in the order the type declares its variants: `{}` before `{}`",
            self.arm, self.before
        )
    }
}

/// One constructor an arm writes: where its type declares it, and what it is called.
struct Step {
    at: usize,
    name: String,
}

/// Why `written` belongs above `above`, which is written above it, when it does.
///
/// The two are read together for as long as both are placed, which is what makes an arm that
/// reaches inside a variant answer to the variant first and to what it reaches for second.
fn out_of_place(written: &[Step], above: &[Step], span: Span) -> Option<OutOfOrder> {
    for (here, there) in written.iter().zip(above) {
        if here.at > there.at {
            return None;
        }
        if here.at < there.at {
            return Some(OutOfOrder {
                arm: here.name.clone(),
                before: there.name.clone(),
                span,
            });
        }
    }
    None
}

/// Where every constructor `pattern` writes is declared, outermost first.
fn steps(pattern: &Pattern, reading: &Reading, space: &Space) -> Vec<Step> {
    let mut placed = Vec::new();
    walk(pattern, reading, space, &mut placed);
    placed
}

/// Adds each constructor of `pattern` to `placed`, and says whether it placed all of them.
fn walk(pattern: &Pattern, reading: &Reading, space: &Space, placed: &mut Vec<Step>) -> bool {
    let Some(path) = constructor(pattern, reading) else {
        return false;
    };
    let name = path.to_string();
    let Some(at) = space.declared_at(&name) else {
        return false;
    };
    placed.push(Step { at, name });
    let PatternKind::Tuple { elements, .. } = &pattern.kind else {
        return true;
    };
    elements
        .iter()
        .all(|element| walk(element, reading, space, placed))
}

/// The constructor a pattern names, which is nothing when it binds or writes a literal.
fn constructor<'a>(pattern: &'a Pattern, reading: &Reading) -> Option<&'a Path> {
    match &pattern.kind {
        PatternKind::Tuple { path, .. } | PatternKind::Record { path, .. } => Some(path),
        PatternKind::Name(path) => (!reading.binds(path)).then_some(path),
        PatternKind::Integer(_) | PatternKind::String(_) | PatternKind::Bool(_) => None,
    }
}
