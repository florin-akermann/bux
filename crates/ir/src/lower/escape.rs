//! Which record a function builds and never lets go of, which is a record it need not build.
//!
//! `docs/specs/codegen.md` states the rule: a binding written with `:=` whose value is a record
//! literal, and every other mention of which reads one field of it, is kept in locals rather than
//! built. A Lumen value has no identity, so the two are the same value and only the instructions
//! differ. A mention this walk does not understand is a mention that lets the value go, because a
//! record built where a whole one is wanted is only slower, and one missed is wrong.

use std::collections::HashSet;

use lumen_ast::{Block, Expr, ExprKind, ForHeader, ForLoop, Function, IfExpr, MatchExpr};
use lumen_ast::{Mutability, Name, Span, Statement, StatementKind};
use lumen_resolver::{Definition, DefinitionKind, Namespace, Origin, ResolvedProgram};

use crate::lower::shape::Shapes;

/// The bindings of `function` kept in locals, by the span of the name that declares each.
pub(crate) fn split(
    resolved: &ResolvedProgram,
    shapes: &Shapes,
    function: &Function,
) -> HashSet<Span> {
    let mut walk = Walk {
        resolved,
        shapes,
        built: Vec::new(),
        escaped: HashSet::new(),
    };
    walk.block(&function.body);
    let Walk { built, escaped, .. } = walk;
    built
        .into_iter()
        .filter(|at| !escaped.contains(at))
        .collect()
}

/// One function body part-way through being walked.
struct Walk<'a> {
    resolved: &'a ResolvedProgram,
    shapes: &'a Shapes,
    /// The bindings that could be split: written with `:=`, holding a record literal.
    built: Vec<Span>,
    /// The bindings some mention let go of, by the span of the name that declares each.
    escaped: HashSet<Span>,
}

impl Walk<'_> {
    fn block(&mut self, block: &Block) {
        for statement in &block.statements {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Binding {
                mutability,
                name,
                value,
            } => self.bound(*mutability, name, value),
            StatementKind::Assign { target, value, .. } => {
                self.let_go_of(target);
                self.expr(value);
            }
            StatementKind::Return(value) => self.expr_of(value.as_ref()),
            StatementKind::For(repeated) => self.repeated(repeated),
            StatementKind::Discard(value) | StatementKind::Expr(value) => self.expr(value),
            StatementKind::Break | StatementKind::Continue => {}
        }
    }

    /// A binding that could be split is one written with `:=` holding a record literal of its own.
    fn bound(&mut self, mutability: Mutability, name: &Name, value: &Expr) {
        self.expr(value);
        let literal =
            matches!(&value.kind, ExprKind::Record { base, .. } if self.builds_a_record(base));
        if literal && mutability == Mutability::Immutable {
            self.built.push(name.span);
        }
    }

    fn repeated(&mut self, repeated: &ForLoop) {
        match &repeated.header {
            ForHeader::Forever => {}
            ForHeader::While(condition) => self.expr(condition),
            ForHeader::In { iterable, .. } => self.expr(iterable),
        }
        self.block(&repeated.body);
    }

    fn expr_of(&mut self, expr: Option<&Expr>) {
        if let Some(written) = expr {
            self.expr(written);
        }
    }

    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Name(name) => self.let_go_of(name),
            ExprKind::Field { receiver, .. } => self.read(receiver),
            ExprKind::Unary { operand, .. } => self.expr(operand),
            ExprKind::Binary { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            ExprKind::Call { callee, arguments } => {
                self.expr(callee);
                for value in arguments.values() {
                    self.expr(value);
                }
            }
            ExprKind::Try(inner) => self.expr(inner),
            ExprKind::Record { base, fields } => {
                self.let_go_of(base);
                for field in fields {
                    self.expr(&field.value);
                }
            }
            ExprKind::If(chain) => self.chain(chain),
            ExprKind::Match(matched) => self.matched(matched),
            ExprKind::Integer(_) | ExprKind::String(_) | ExprKind::Bool(_) | ExprKind::Unit => {}
        }
    }

    /// The receiver of a field read is the one mention that does not let the value go: it is read
    /// where it stands, and a record kept in locals has that field in one of them.
    ///
    /// A receiver is read and never passed, because version 0.1 declares no method for a record to
    /// be the receiver of: the lowering refuses any callee that is not a name outright.
    fn read(&mut self, receiver: &Expr) {
        if !matches!(receiver.kind, ExprKind::Name(_)) {
            self.expr(receiver);
        }
    }

    fn chain(&mut self, chain: &IfExpr) {
        for branch in &chain.branches {
            self.expr(&branch.condition);
            self.block(&branch.block);
        }
        if let Some(block) = &chain.otherwise {
            self.block(block);
        }
    }

    fn matched(&mut self, matched: &MatchExpr) {
        self.expr(&matched.scrutinee);
        for arm in &matched.arms {
            self.expr(&arm.body);
        }
    }

    /// Records that whatever `name` means is wanted whole, so a record it means has to be built.
    fn let_go_of(&mut self, name: &Name) {
        if let Some(at) = self.declares(name) {
            self.escaped.insert(at);
        }
    }

    /// Whether `base` names the constructor of the record type a literal builds, rather than a
    /// binding the literal updates or a variant of an algebraic data type.
    ///
    /// A variant carries a tag its constructor writes, so splitting one would lose which variant
    /// the value is; only a record is nothing but the fields it declares.
    fn builds_a_record(&self, base: &Name) -> bool {
        self.definition(base)
            .is_some_and(|definition| definition.kind == DefinitionKind::Constructor)
            && self.shapes.builds_a_record(&base.text)
    }

    /// Where the binding `name` means was declared, when it means a binding at all.
    fn declares(&self, name: &Name) -> Option<Span> {
        match self.definition(name)?.origin {
            Origin::Declared(at) => Some(at),
            Origin::Prelude => None,
        }
    }

    fn definition(&self, name: &Name) -> Option<Definition> {
        self.resolved.definition(Namespace::Value, name)
    }
}
