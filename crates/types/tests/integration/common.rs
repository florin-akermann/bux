//! Helpers shared by the type inference behaviour tests.
//!
//! A behaviour is stated as a whole module, because that is what inference takes. The type of one
//! expression of it is then read back by the text that expression is written with.

use lumen_ast::{Block, Expr, ExprKind, ForHeader, IfExpr, Program, Span};
use lumen_ast::{Statement, StatementKind};
use lumen_parser::parse;
use lumen_resolver::{ResolvedProgram, library, resolve};
use lumen_types::{Imported, TypeError, TypedProgram, check};

/// The name the module under test is compiled as, which nothing here depends on.
const MODULE: &str = "demo";

/// The failure `source` is refused with, importing nothing.
pub fn refusal(source: &str) -> TypeError {
    refusal_reaching(source, &Imported::default())
}

/// The failure `source` is refused with, reaching what `imported` offers.
pub fn refusal_reaching(source: &str, imported: &Imported) -> TypeError {
    check(resolved(source), imported)
        .err()
        .unwrap_or_else(|| panic!("{source:?} is refused"))
}

/// A module the source under test imports, which inference reads the surface of.
pub struct Offered<'w> {
    pub module: &'w str,
    pub source: &'w str,
}

/// What `offered` puts out, under the name the source under test imports it by.
pub fn offering(offered: &Offered<'_>) -> Imported {
    let surface = inferred(offered.source).surface().clone();
    Imported::default().offering(offered.module, surface)
}

/// What every library module puts out, which is what a module importing one of them reaches.
///
/// They are read out of the source the compiler carries, so inference is given their surfaces
/// exactly as it is given a loaded module's. Each is read reaching the ones before it, which is
/// the order the loader hands them over in, and is what `files` and `process` need: each imports
/// `list`, and `library::carried()` hands `list` over first.
pub fn reaching_the_library() -> Imported {
    library::carried()
        .filter(|(module, _)| *module != library::PRELUDE)
        .fold(Imported::default(), |imported, (module, source)| {
            let surface = inferred_reaching(source, &imported).surface().clone();
            imported.offering(module, surface)
        })
}

/// The type of the occurrence of `written` numbered `occurrence`, counting from one.
pub fn inferred_type(source: &str, written: &str, occurrence: usize) -> String {
    inferred_type_reaching(source, written, occurrence, &Imported::default())
}

/// The same, of a module reaching what `imported` offers.
pub fn inferred_type_reaching(
    source: &str,
    written: &str,
    occurrence: usize,
    imported: &Imported,
) -> String {
    let at = source
        .match_indices(written)
        .nth(occurrence - 1)
        .unwrap_or_else(|| panic!("{source:?} writes {written:?} {occurrence} times"))
        .0;
    inferred_reaching(source, imported)
        .type_of(Span::new(at, written.len()))
        .unwrap_or_else(|| panic!("{written:?} number {occurrence} is an expression"))
        .to_string()
}

/// The inferred program of `source`, which must infer, importing nothing.
pub fn inferred(source: &str) -> TypedProgram {
    inferred_reaching(source, &Imported::default())
}

/// The inferred program of `source`, which must infer, reaching what `imported` offers.
pub fn inferred_reaching(source: &str, imported: &Imported) -> TypedProgram {
    check(resolved(source), imported)
        .unwrap_or_else(|error| panic!("{source:?} infers: {}", error.message()))
}

/// Every expression `program` writes, each one exactly once.
///
/// An instance's method has a body like any other function's, so its expressions are among them.
pub fn expressions(program: &Program) -> Vec<&Expr> {
    let mut found = Vec::new();
    for function in program.functions() {
        from_block(&function.body, &mut found);
    }
    found
}

fn resolved(source: &str) -> ResolvedProgram {
    let program = parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {error:?}"));
    resolve(program, MODULE)
        .unwrap_or_else(|error| panic!("{source:?} resolves: {}", error.message()))
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
        StatementKind::Discard(expr) | StatementKind::Expr(expr) => from_expr(expr, into),
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
            for argument in arguments.values() {
                from_expr(argument, into);
            }
        }
        ExprKind::Field { receiver, .. } => from_expr(receiver, into),
        ExprKind::Try(inner) => from_expr(inner, into),
        ExprKind::List(elements) => {
            for element in elements {
                from_expr(element, into);
            }
        }
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
