//! Which trait each operator is, and what an operator gives back.
//!
//! `docs/specs/operators.md` states both. An operator is a call of its trait's method, so what
//! inference does with one is what it does with any call: it holds the two sides to one type,
//! asks the trait of that type, and gives back what the trait's method gives back.

use lumen_ast::{BinaryOperator, Expr, ExprKind, Span, UnaryOperator};
use lumen_resolver::prelude;

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::{Binary, Inference};
use crate::types::Type;

impl Inference<'_> {
    /// `!` asks nothing of a type, and prefix `-` asks `Neg` of the one it is written over.
    pub(crate) fn unary(
        &mut self,
        operator: UnaryOperator,
        operand: &Expr,
        at: Span,
    ) -> Result<Type, TypeError> {
        let found = self.expr(operand)?;
        if operator == UnaryOperator::Not {
            self.expect(&Type::boolean(), &found, operand.span)?;
            return Ok(Type::boolean());
        }
        self.operated.push(Operated {
            of: prelude::NEG,
            written_as: "-",
            at: found.clone(),
            written: at,
        });
        Ok(found)
    }

    /// Both sides of an operator are one type, and the operator's trait is asked of it.
    ///
    /// `&&` and `||` are the exception `docs/specs/operators.md` names: each decides whether to
    /// run the other side, which no call can, so they stay `Bool`'s alone.
    pub(crate) fn binary(&mut self, written: &Binary<'_>) -> Result<Type, TypeError> {
        let Binary {
            operator,
            left,
            right,
            at,
        } = *written;
        let found = self.expr(left)?;
        let other = self.expr(right)?;
        let Some(asked) = asked_by(operator) else {
            self.expect(&Type::boolean(), &found, left.span)?;
            self.expect(&Type::boolean(), &other, right.span)?;
            return Ok(Type::boolean());
        };
        self.expect(&found, &other, right.span)?;
        if asked.of == prelude::DIV || asked.of == prelude::REM {
            divides_by_zero(right)?;
        }
        self.operated.push(Operated {
            of: asked.of,
            written_as: asked.written_as,
            at: found.clone(),
            written: at,
        });
        Ok(gives_back(operator, found))
    }

    /// `+=` adds, so it asks `Add` of the type of what is being assigned to.
    pub(crate) fn adds_to(&mut self, assigned: Type, at: Span) {
        self.operated.push(Operated {
            of: prelude::ADD,
            written_as: "+=",
            at: assigned,
            written: at,
        });
    }
}

/// One operator waiting on the type its operands settled on, which its trait is asked of.
pub(crate) struct Operated {
    /// The trait the operator is, which `docs/specs/operators.md` names.
    pub(crate) of: &'static str,
    /// What the reader wrote, which is how a type with no instance of `of` is refused.
    pub(crate) written_as: &'static str,
    pub(crate) at: Type,
    /// The whole expression, which is what a refused operator points the reader at.
    pub(crate) written: Span,
}

/// The trait an operator is, and how it is written, or nothing where it is `Bool`'s alone.
const fn asked_by(operator: BinaryOperator) -> Option<Operator> {
    let asked = match operator {
        BinaryOperator::Or | BinaryOperator::And => return None,
        BinaryOperator::Equal => Operator::new(prelude::EQ, "=="),
        BinaryOperator::NotEqual => Operator::new(prelude::EQ, "!="),
        BinaryOperator::Less => Operator::new(prelude::ORD, "<"),
        BinaryOperator::LessOrEqual => Operator::new(prelude::ORD, "<="),
        BinaryOperator::Greater => Operator::new(prelude::ORD, ">"),
        BinaryOperator::GreaterOrEqual => Operator::new(prelude::ORD, ">="),
        BinaryOperator::Add => Operator::new(prelude::ADD, "+"),
        BinaryOperator::Subtract => Operator::new(prelude::SUB, "-"),
        BinaryOperator::Multiply => Operator::new(prelude::MUL, "*"),
        BinaryOperator::Divide => Operator::new(prelude::DIV, "/"),
        BinaryOperator::Remainder => Operator::new(prelude::REM, "%"),
    };
    Some(asked)
}

/// What an operator gives back, which is what its trait's method gives back.
fn gives_back(operator: BinaryOperator, found: Type) -> Type {
    match operator {
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::Less
        | BinaryOperator::LessOrEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterOrEqual => Type::boolean(),
        BinaryOperator::Divide | BinaryOperator::Remainder => Type::option(found),
        BinaryOperator::Or
        | BinaryOperator::And
        | BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply => found,
    }
}

/// Refuses a division by a `0` the compiler can already see, which has no answer to give.
fn divides_by_zero(right: &Expr) -> Result<(), TypeError> {
    if right.kind == ExprKind::Integer(0) {
        return Err(TypeError::at(right.span, TypeErrorKind::DivisorIsZero));
    }
    Ok(())
}

/// The trait one operator is, beside the way the reader wrote it.
struct Operator {
    of: &'static str,
    written_as: &'static str,
}

impl Operator {
    const fn new(of: &'static str, written_as: &'static str) -> Self {
        Self { of, written_as }
    }
}
