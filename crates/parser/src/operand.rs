//! One operand: a value, and the calls, fields, and `?` applied to it.
//!
//! An operator's operand lives here rather than in `expr`, so that the precedence chain reads
//! top down in one file and the recursion back into an expression crosses a module boundary.

use lumen_ast::{Expr, ExprKind};
use lumen_lexer::{Keyword, Punct, TokenKind};

use crate::control::{if_expression, match_expression};
use crate::cursor::Cursor;
use crate::error::{Expected, ParseError};
use crate::expr::{RecordLiterals, expression};
use crate::list::{Emptiness, comma_separated};
use crate::literal::{Literal, eat_literal};
use crate::record::record_literal;

pub(crate) fn postfix(cursor: &mut Cursor, records: RecordLiterals) -> Result<Expr, ParseError> {
    let start = cursor.offset();
    let base = primary(cursor, records)?;
    postfix_chain(cursor, base, start)
}

/// A call, a field, or a `?`, applied left to right to whatever came before it.
pub(crate) fn postfix_chain(
    cursor: &mut Cursor,
    base: Expr,
    start: usize,
) -> Result<Expr, ParseError> {
    let mut expr = base;
    loop {
        let kind = if cursor.eat_punct(Punct::LParen).is_some() {
            ExprKind::Call {
                callee: Box::new(expr),
                arguments: arguments(cursor)?,
            }
        } else if cursor.eat_punct(Punct::Dot).is_some() {
            ExprKind::Field {
                receiver: Box::new(expr),
                name: cursor.expect_name(Expected::Name)?,
            }
        } else if cursor.eat_punct(Punct::Question).is_some() {
            ExprKind::Try(Box::new(expr))
        } else {
            return Ok(expr);
        };
        expr = Expr {
            kind,
            span: cursor.span_since(start),
        };
    }
}

fn arguments(cursor: &mut Cursor) -> Result<Vec<Expr>, ParseError> {
    comma_separated(cursor, Punct::RParen, Emptiness::Allowed, |cursor| {
        expression(cursor, RecordLiterals::Allowed)
    })
}

/// One value, and the level the nesting budget is spent at.
///
/// Every nested expression reaches a `primary` of its own, whether the nesting is written with
/// parentheses, arguments, record fields, or the blocks of an `if` or a `match`.
fn primary(cursor: &mut Cursor, records: RecordLiterals) -> Result<Expr, ParseError> {
    let start = cursor.offset();
    let kind = cursor.nested(|cursor| primary_kind(cursor, records))?;
    Ok(Expr {
        kind,
        span: cursor.span_since(start),
    })
}

fn primary_kind(cursor: &mut Cursor, records: RecordLiterals) -> Result<ExprKind, ParseError> {
    if let Some(literal) = eat_literal(cursor)? {
        return Ok(of_literal(literal));
    }
    match cursor.peek_kind(0) {
        Some(TokenKind::Keyword(Keyword::If)) => Ok(ExprKind::If(Box::new(if_expression(cursor)?))),
        Some(TokenKind::Keyword(Keyword::Match)) => {
            Ok(ExprKind::Match(Box::new(match_expression(cursor)?)))
        }
        Some(TokenKind::Punct(Punct::LParen)) => parenthesised(cursor),
        Some(TokenKind::Identifier) => name_or_record(cursor, records),
        _ => Err(cursor.error(Expected::Expression)),
    }
}

fn of_literal(literal: Literal) -> ExprKind {
    match literal {
        Literal::Integer(value) => ExprKind::Integer(value),
        Literal::String(value) => ExprKind::String(value),
        Literal::Bool(value) => ExprKind::Bool(value),
    }
}

/// `()`, or an expression in parentheses, which takes the parentheses into its span.
fn parenthesised(cursor: &mut Cursor) -> Result<ExprKind, ParseError> {
    cursor.expect_punct(Punct::LParen)?;
    if cursor.eat_punct(Punct::RParen).is_some() {
        return Ok(ExprKind::Unit);
    }
    let inner = expression(cursor, RecordLiterals::Allowed)?;
    cursor.expect_punct(Punct::RParen)?;
    Ok(inner.kind)
}

/// A name, or the record literal it heads.
fn name_or_record(cursor: &mut Cursor, records: RecordLiterals) -> Result<ExprKind, ParseError> {
    let base = cursor.expect_name(Expected::Expression)?;
    if records == RecordLiterals::Forbidden || !cursor.at_punct(Punct::LBrace) {
        return Ok(ExprKind::Name(base));
    }
    record_literal(cursor, base)
}
