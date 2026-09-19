//! Every hole of a module, found where it is written.

use lumen_ast::{Block, Branch, Expr, ExprKind, ForHeader, ForLoop, IfExpr, Item, MatchExpr, Name};
use lumen_ast::{Statement, StatementKind};
use lumen_resolver::{Namespace, Origin, ResolvedProgram};

use crate::hole::Hole;

/// What the prelude calls the hole, which `docs/specs/modules.md` lists among its names.
const TODO: &str = "todo";

/// Every hole of `resolved`, in the order they are written.
pub(crate) fn module(resolved: &ResolvedProgram) -> Vec<Hole> {
    let walk = Walk { resolved };
    let mut found = Vec::new();
    for item in &resolved.program().items {
        if let Item::Function(function) = item {
            walk.block(&function.body, &mut found);
        }
    }
    found
}

/// What a walk carries: what each name means, which is what tells a hole from a call.
struct Walk<'a> {
    resolved: &'a ResolvedProgram,
}

impl Walk<'_> {
    fn block(&self, block: &Block, found: &mut Vec<Hole>) {
        for statement in &block.statements {
            self.statement(statement, found);
        }
    }

    fn statement(&self, statement: &Statement, found: &mut Vec<Hole>) {
        match &statement.kind {
            StatementKind::Binding { value, .. }
            | StatementKind::Assign { value, .. }
            | StatementKind::Discard(value)
            | StatementKind::Expr(value) => self.expr(value, found),
            StatementKind::Return(returned) => {
                if let Some(value) = returned {
                    self.expr(value, found);
                }
            }
            StatementKind::Break | StatementKind::Continue => {}
            StatementKind::For(repeated) => self.for_loop(repeated, found),
        }
    }

    fn for_loop(&self, repeated: &ForLoop, found: &mut Vec<Hole>) {
        match &repeated.header {
            ForHeader::Forever => {}
            ForHeader::While(condition) => self.expr(condition, found),
            ForHeader::In { iterable, .. } => self.expr(iterable, found),
        }
        self.block(&repeated.body, found);
    }

    fn expr(&self, expr: &Expr, found: &mut Vec<Hole>) {
        match &expr.kind {
            ExprKind::Name(_)
            | ExprKind::Integer(_)
            | ExprKind::String(_)
            | ExprKind::Bool(_)
            | ExprKind::Unit => {}
            ExprKind::Unary { operand, .. } => self.expr(operand, found),
            ExprKind::Binary { left, right, .. } => {
                self.expr(left, found);
                self.expr(right, found);
            }
            ExprKind::Call { .. } => self.call(expr, found),
            ExprKind::Field { receiver, .. } => self.expr(receiver, found),
            ExprKind::Try(inner) => self.expr(inner, found),
            ExprKind::Record { fields, .. } => {
                for field in fields {
                    self.expr(&field.value, found);
                }
            }
            ExprKind::If(chain) => self.if_expr(chain, found),
            ExprKind::Match(matched) => self.match_expr(matched, found),
        }
    }

    /// A call, which is a hole when it is the prelude's `todo` and an ordinary call otherwise.
    ///
    /// The arguments are walked either way, because `todo(name_of(todo("why")))` holds two.
    fn call(&self, whole: &Expr, found: &mut Vec<Hole>) {
        let ExprKind::Call { callee, arguments } = &whole.kind else {
            unreachable!("a call is what reaches here")
        };
        match &callee.kind {
            ExprKind::Name(name) if self.is_the_hole(name) => found.push(Hole::at(whole.span)),
            _ => self.expr(callee, found),
        }
        for argument in arguments.values() {
            self.expr(argument, found);
        }
    }

    /// Whether `name` is the hole the prelude supplies, rather than one a module declared.
    fn is_the_hole(&self, name: &Name) -> bool {
        name.text == TODO
            && self
                .resolved
                .definition(Namespace::Value, name)
                .is_some_and(|definition| definition.origin == Origin::Prelude)
    }

    fn if_expr(&self, chain: &IfExpr, found: &mut Vec<Hole>) {
        for branch in &chain.branches {
            self.branch(branch, found);
        }
        if let Some(otherwise) = &chain.otherwise {
            self.block(otherwise, found);
        }
    }

    fn branch(&self, branch: &Branch, found: &mut Vec<Hole>) {
        self.expr(&branch.condition, found);
        self.block(&branch.block, found);
    }

    fn match_expr(&self, matched: &MatchExpr, found: &mut Vec<Hole>) {
        self.expr(&matched.scrutinee, found);
        for arm in &matched.arms {
            self.expr(&arm.body, found);
        }
    }
}
