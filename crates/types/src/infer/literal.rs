//! What a whole-number literal is, which is the method of the trait a literal is.
//!
//! `docs/specs/literals.md` states it: a literal's type is a variable standing for a whole number,
//! which settles only on a type with an instance of `IntegerLiteral` and is an `Int` when nothing
//! settles it. Unification holds it to that much, so what is left here is the range: the number
//! itself is held to the bounds the instance of the type it settled on states.

use std::mem;

use lumen_ast::Span;

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::Inference;
use crate::types::Type;

impl Inference<'_> {
    /// A whole number, whose type is a variable until the code around it says which type it is.
    pub(crate) fn literal(&mut self, value: i64, written: Span) -> Type {
        let at = self.table.fresh_whole_number();
        self.literals.push(Written {
            value,
            at: at.clone(),
            span: written,
        });
        at
    }

    /// The whole numbers of one function, each settled and then held to the range it says.
    ///
    /// They settle first of all, because every other thing that waits on a type waits on one a
    /// literal may be standing in: the type of `1 + 1` is the type of its literals. One nothing
    /// settled is an `Int` from here on, so everything after this reads a type rather than a wait.
    pub(crate) fn settle_literals(&mut self) -> Result<(), TypeError> {
        for literal in mem::take(&mut self.literals) {
            if matches!(self.table.shallow(&literal.at), Type::Var(_)) {
                self.expect(&Type::int(), &literal.at, literal.span)?;
            }
            let at = self.table.solved(&literal.at);
            let Type::Named { name, .. } = &at else {
                unreachable!(
                    "a whole number settles on a named type, which unification holds it to"
                )
            };
            let Some(bounds) = self.environment.holds(name) else {
                unreachable!("`{name}` has the instance, which is why a whole number settled on it")
            };
            if bounds.holds(literal.value) {
                continue;
            }
            let kind = TypeErrorKind::LiteralDoesNotFit {
                value: literal.value,
                at,
                lowest: bounds.lowest,
                highest: bounds.highest,
            };
            return Err(TypeError::at(literal.span, kind));
        }
        Ok(())
    }
}

impl Written {
    /// The type this whole number is waiting on, which holds its variable until it settles.
    pub(crate) const fn at(&self) -> &Type {
        &self.at
    }
}

/// One whole number waiting on the type the code around it settles.
pub(crate) struct Written {
    /// The number as the lexer read it, which is what the bounds are asked about.
    value: i64,
    at: Type,
    /// Where it is written, which is what a refusal points the reader at.
    span: Span,
}
