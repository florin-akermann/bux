//! Helpers shared by the type inference behaviour tests.
//!
//! A behaviour is stated as a whole module, because that is what inference takes. The type of one
//! expression of it is then read back by the text that expression is written with.

use lumen_ast::{Block, Expr, ExprKind, ForHeader, IfExpr, Item, Program, Span};
use lumen_ast::{Statement, StatementKind};
use lumen_parser::parse;
use lumen_resolver::{ResolvedProgram, resolve};
use lumen_types::{TypeError, TypedProgram, check};

/// The failure `source` is refused with.
pub fn refusal(source: &str) -> TypeError {
    check(resolved(source))
        .err()
        .unwrap_or_else(|| panic!("{source:?} is refused"))
}

/// The type of the occurrence of `written` numbered `occurrence`, counting from one.
pub fn inferred_type(source: &str, written: &str, occurrence: usize) -> String {
    let at = source
        .match_indices(written)
        .nth(occurrence - 1)
        .unwrap_or_else(|| panic!("{source:?} writes {written:?} {occurrence} times"))
        .0;
    inferred(source)
        .type_of(Span::new(at, written.len()))
        .unwrap_or_else(|| panic!("{written:?} number {occurrence} is an expression"))
        .to_string()
}

/// The inferred program of `source`, which must infer.
pub fn inferred(source: &str) -> TypedProgram {
    check(resolved(source)).unwrap_or_else(|error| panic!("{source:?} infers: {}", error.message()))
}

/// Every expression `program` writes, each one exactly once.
pub fn expressions(program: &Program) -> Vec<&Expr> {
    let mut found = Vec::new();
    for item in &program.items {
        if let Item::Function(function) = item {
            from_block(&function.body, &mut found);
        }
    }
    found
}

fn resolved(source: &str) -> ResolvedProgram {
    let program = parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"));
    resolve(program).unwrap_or_else(|error| panic!("{source:?} resolves: {}", error.message()))
}
fn from_block<'a>(block: &'a Block, into: &mut Vec<&'a Expr>) {
    for statement in &block.statements {
        from_statement(statement, into);
    }
}

fn from_statement<'a>(statement: &'a Statement, into: &mut Vec<&'a Expr>) {
    match &statement.kind {
        StatementKind::Binding { value, .. } | StatementKind::Assign { value, .. } => {
            from_expr(value, into);
        }
        StatementKind::Return(value) => {
            if let Some(value) = value {
                from_expr(value, into);
            }
        }
        StatementKind::Break | StatementKind::Continue => {}
        StatementKind::For(walked) => {
            match &walked.header {
                ForHeader::Forever => {}
                ForHeader::While(condition) => from_expr(condition, into),
                ForHeader::In { iterable, .. } => from_expr(iterable, into),
            }
            from_block(&walked.body, into);
        }
        StatementKind::Expr(expr) => from_expr(expr, into),
    }
}

fn from_expr<'a>(expr: &'a Expr, into: &mut Vec<&'a Expr>) {
    into.push(expr);
    match &expr.kind {
        ExprKind::Name(_)
        | ExprKind::Integer(_)
        | ExprKind::String(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit => {}
        ExprKind::Unary { operand, .. } => from_expr(operand, into),
        ExprKind::Binary { left, right, .. } => {
            from_expr(left, into);
            from_expr(right, into);
        }
        ExprKind::Call { callee, arguments } => {
            from_expr(callee, into);
            for argument in arguments {
                from_expr(argument, into);
            }
        }
        ExprKind::Field { receiver, .. } => from_expr(receiver, into),
        ExprKind::Try(inner) => from_expr(inner, into),
        ExprKind::Record { fields, .. } => {
            for field in fields {
                from_expr(&field.value, into);
            }
        }
        ExprKind::If(chain) => from_if(chain, into),
        ExprKind::Match(matched) => {
            from_expr(&matched.scrutinee, into);
            for arm in &matched.arms {
                from_expr(&arm.body, into);
            }
        }
    }
}

fn from_if<'a>(chain: &'a IfExpr, into: &mut Vec<&'a Expr>) {
    for branch in &chain.branches {
        from_expr(&branch.condition, into);
        from_block(&branch.block, into);
    }
    if let Some(block) = &chain.otherwise {
        from_block(block, into);
    }
}
