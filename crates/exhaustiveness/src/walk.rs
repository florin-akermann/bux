//! Every `match` of a module, checked where it is written.

use lumen_ast::Program;
use lumen_ast::{Block, Expr, ExprKind, ForHeader, ForLoop, IfExpr, Item, MatchExpr};
use lumen_ast::{Span, Statement, StatementKind};

use crate::error::MatchError;
use crate::pattern::Reading;
use crate::space::Space;
use crate::usefulness::uncovered;

/// Either every `match` answers for every value, or this is the first one that does not.
type Checked = Result<(), MatchError>;

/// Checks every `match` of `program`, in the order they are written.
pub(crate) fn module(program: &Program, reading: &Reading, space: &Space) -> Checked {
    let walk = Walk { reading, space };
    for item in &program.items {
        if let Item::Function(function) = item {
            walk.block(&function.body)?;
        }
    }
    Ok(())
}

/// What a walk carries: what the patterns mean, and what they have to cover.
struct Walk<'a> {
    reading: &'a Reading<'a>,
    space: &'a Space,
}

impl Walk<'_> {
    fn block(&self, block: &Block) -> Checked {
        for statement in &block.statements {
            self.statement(statement)?;
        }
        Ok(())
    }

    fn statement(&self, statement: &Statement) -> Checked {
        match &statement.kind {
            StatementKind::Binding { value, .. } | StatementKind::Expr(value) => self.expr(value),
            StatementKind::Assign { target, value, .. } => {
                self.expr(target)?;
                self.expr(value)
            }
            StatementKind::Return(returned) => match returned {
                Some(value) => self.expr(value),
                None => Ok(()),
            },
            StatementKind::Break | StatementKind::Continue => Ok(()),
            StatementKind::For(repeated) => self.for_loop(repeated),
        }
    }

    fn for_loop(&self, repeated: &ForLoop) -> Checked {
        match &repeated.header {
            ForHeader::Forever => Ok(()),
            ForHeader::While(condition) => self.expr(condition),
            ForHeader::In { iterable, .. } => self.expr(iterable),
        }?;
        self.block(&repeated.body)
    }

    fn expr(&self, expr: &Expr) -> Checked {
        match &expr.kind {
            ExprKind::Name(_)
            | ExprKind::Integer(_)
            | ExprKind::String(_)
            | ExprKind::Bool(_)
            | ExprKind::Unit => Ok(()),
            ExprKind::Unary { operand, .. } => self.expr(operand),
            ExprKind::Try(inner)
            | ExprKind::Field {
                receiver: inner, ..
            } => self.expr(inner),
            ExprKind::Binary { left, right, .. } => {
                self.expr(left)?;
                self.expr(right)
            }
            ExprKind::Call { callee, arguments } => {
                self.expr(callee)?;
                self.each(arguments.iter())
            }
            ExprKind::Record { fields, .. } => self.each(fields.iter().map(|field| &field.value)),
            ExprKind::If(branching) => self.if_expr(branching),
            ExprKind::Match(matching) => self.match_expr(matching, expr.span),
        }
    }

    fn if_expr(&self, branching: &IfExpr) -> Checked {
        for branch in &branching.branches {
            self.expr(&branch.condition)?;
            self.block(&branch.block)?;
        }
        match &branching.otherwise {
            Some(block) => self.block(block),
            None => Ok(()),
        }
    }

    fn match_expr(&self, matching: &MatchExpr, at: Span) -> Checked {
        self.expr(&matching.scrutinee)?;
        let rows: Vec<Vec<_>> = matching
            .arms
            .iter()
            .map(|arm| vec![self.reading.of(&arm.pattern)])
            .collect();
        let left = uncovered(self.space, &rows, 1);
        if !left.is_empty() {
            return Err(MatchError::at(at, &left));
        }
        self.each(matching.arms.iter().map(|arm| &arm.body))
    }

    fn each<'e>(&self, expressions: impl Iterator<Item = &'e Expr>) -> Checked {
        for expr in expressions {
            self.expr(expr)?;
        }
        Ok(())
    }
}
