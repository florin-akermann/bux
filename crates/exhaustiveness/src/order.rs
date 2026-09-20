//! The order a `match` lists its arms in, which is the order the type declares its variants.
//!
//! `docs/design.md` section 13 asks for it so that a new variant has exactly one place to be
//! handled and no diff is ever reorder-only. The rule is checked here because this is the phase
//! that knows the variant list; the order is said, never rewritten.
//!
//! An arm is placed by the constructors it writes, read left to right and outermost first, so
//! two arms that reach inside one variant are placed by what they reach for. Placing stops at
//! the first thing no declaration writes down: a name that binds, `_`, a number, a string, and
//! the two values of a `Bool` are each written where the author put them, and so is everything
//! an arm writes after one of them.
//!
//! An or-pattern is placed by its first alternative, and its alternatives are held to the order
//! among themselves that arms are held to. `docs/specs/patterns.md` states both: an arm that
//! answers for two variants sits where the first of them is declared, and `Running | Pending` is
//! refused wherever `Running => …` above `Pending => …` would be.

use lumen_ast::{MatchExpr, Path, Pattern, PatternKind, Span};

use crate::pattern::Reading;

/// What a pattern is placed against: which names bind, and where each constructor is declared.
pub(crate) struct Placing<'a> {
    reading: &'a Reading<'a>,
}

impl<'a> Placing<'a> {
    pub(crate) const fn new(reading: &'a Reading<'a>) -> Self {
        Self { reading }
    }

    /// The first arm of `matching` written above one the declarations put it below, when one is.
    ///
    /// An or-pattern's own alternatives are read first, because an arm placed by one of them is
    /// placed by a `|` the author is about to be told to rewrite anyway.
    pub(crate) fn out_of_order(&self, matching: &MatchExpr) -> Option<OutOfOrder> {
        let arms: Vec<&Pattern> = matching.arms.iter().map(|arm| &arm.pattern).collect();
        arms.iter()
            .find_map(|arm| self.alternatives_of(arm))
            .or_else(|| self.first_out_of_place(&arms))
    }

    /// The alternatives of every or-pattern inside `pattern`, held to the order arms are held to.
    fn alternatives_of(&self, pattern: &Pattern) -> Option<OutOfOrder> {
        match &pattern.kind {
            PatternKind::Or(alternatives) => {
                let written: Vec<&Pattern> = alternatives.iter().collect();
                self.first_out_of_place(&written)
                    .or_else(|| written.iter().find_map(|one| self.alternatives_of(one)))
            }
            PatternKind::Tuple { elements, .. } => elements
                .iter()
                .find_map(|element| self.alternatives_of(element)),
            _ => None,
        }
    }

    /// The first of `written` placed above one the declarations put it below, when one is.
    fn first_out_of_place(&self, written: &[&Pattern]) -> Option<OutOfOrder> {
        let placed: Vec<Vec<Step>> = written.iter().map(|pattern| self.steps(pattern)).collect();
        for (at, one) in placed.iter().enumerate() {
            let span = written[at].span;
            let found = placed[..at]
                .iter()
                .find_map(|above| out_of_place(one, above, span));
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// Where every constructor `pattern` writes is declared, outermost first.
    fn steps(&self, pattern: &Pattern) -> Vec<Step> {
        let mut placed = Vec::new();
        self.walk(pattern, &mut placed);
        placed
    }

    /// Adds each constructor of `pattern` to `placed`, and says whether it placed all of them.
    fn walk(&self, pattern: &Pattern, placed: &mut Vec<Step>) -> bool {
        if let PatternKind::Or(alternatives) = &pattern.kind {
            let Some(first) = alternatives.first() else {
                return false;
            };
            return self.walk(first, placed);
        }
        let Some(path) = self.constructor(pattern) else {
            return false;
        };
        let name = path.to_string();
        let Some(at) = self.reading.space().declared_at(&name) else {
            return false;
        };
        placed.push(Step { at, name });
        let PatternKind::Tuple { elements, .. } = &pattern.kind else {
            return true;
        };
        elements.iter().all(|element| self.walk(element, placed))
    }

    /// The constructor a pattern names, which is nothing when it binds or writes a literal.
    fn constructor<'p>(&self, pattern: &'p Pattern) -> Option<&'p Path> {
        match &pattern.kind {
            PatternKind::Tuple { path, .. } | PatternKind::Record { path, .. } => Some(path),
            PatternKind::Name(path) => (!self.reading.binds(path)).then_some(path),
            PatternKind::Integer(_)
            | PatternKind::String(_)
            | PatternKind::Bool(_)
            | PatternKind::Wildcard
            | PatternKind::Or(_) => None,
        }
    }
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
