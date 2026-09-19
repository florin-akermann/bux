//! Expressions, by precedence.

use lumen_ast::{BinaryOperator, Expr, ExprKind, UnaryOperator};
use lumen_lexer::{Keyword, Punct, TokenKind};

use crate::control::{if_expression, match_expression};
use crate::cursor::Cursor;
use crate::error::{Expected, ParseError, ParseErrorKind};
use crate::list::comma_separated;
use crate::literal::{Literal, decode_integer, eat_literal};
use crate::record::record_literal;

/// Whether a `{` after a name opens a record literal or the block of an `if`, `for`, or `match`.
///
/// Go resolves the same ambiguity the same way: in those three headers a record literal needs
/// parentheses around it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecordLiterals {
    Allowed,
    Forbidden,
}

/// The infix operators by precedence, loosest level first.
type Level = &'static [(Punct, BinaryOperator)];

const LEVELS: [Level; 5] = [
    &[(Punct::OrOr, BinaryOperator::Or)],
    &[(Punct::AndAnd, BinaryOperator::And)],
    &[
        (Punct::EqEq, BinaryOperator::Equal),
        (Punct::BangEq, BinaryOperator::NotEqual),
        (Punct::Lt, BinaryOperator::Less),
        (Punct::LtEq, BinaryOperator::LessOrEqual),
        (Punct::Gt, BinaryOperator::Greater),
        (Punct::GtEq, BinaryOperator::GreaterOrEqual),
    ],
    &[
        (Punct::Plus, BinaryOperator::Add),
        (Punct::Minus, BinaryOperator::Subtract),
    ],
    &[
        (Punct::Star, BinaryOperator::Multiply),
        (Punct::Slash, BinaryOperator::Divide),
        (Punct::Percent, BinaryOperator::Remainder),
    ],
];

/// The one level that does not chain: `a < b < c` is an error rather than a nested comparison.
const COMPARISON_LEVEL: usize = 2;

pub(crate) fn expression(cursor: &mut Cursor, records: RecordLiterals) -> Result<Expr, ParseError> {
    binary(cursor, records, 0)
}

fn binary(cursor: &mut Cursor, records: RecordLiterals, level: usize) -> Result<Expr, ParseError> {
    let Some(operators) = LEVELS.get(level) else {
        return unary(cursor, records);
    };
    let start = cursor.offset();
    let mut left = binary(cursor, records, level + 1)?;
    while let Some(operator) = eat_operator(cursor, operators) {
        let right = binary(cursor, records, level + 1)?;
        left = Expr {
            kind: ExprKind::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            span: cursor.span_since(start),
        };
        if level == COMPARISON_LEVEL && at_operator(cursor, operators) {
            return Err(cursor.error_kind(ParseErrorKind::ChainedComparison));
        }
    }
    Ok(left)
}

fn eat_operator(cursor: &mut Cursor, operators: Level) -> Option<BinaryOperator> {
    let (punct, operator) = *operators
        .iter()
        .find(|(punct, _)| cursor.at_punct(*punct))?;
    cursor.eat_punct(punct)?;
    Some(operator)
}

fn at_operator(cursor: &Cursor, operators: Level) -> bool {
    operators.iter().any(|(punct, _)| cursor.at_punct(*punct))
}

fn unary(cursor: &mut Cursor, records: RecordLiterals) -> Result<Expr, ParseError> {
    let start = cursor.offset();
    let Some(operator) = eat_unary_operator(cursor) else {
        return postfix(cursor, records);
    };
    let kind = match negated_number(cursor, operator, start)? {
        Some(value) => ExprKind::Integer(value),
        None => ExprKind::Unary {
            operator,
            operand: Box::new(unary(cursor, records)?),
        },
    };
    Ok(Expr {
        kind,
        span: cursor.span_since(start),
    })
}

/// A `-` straight before a number spells one number, so the smallest whole number can be written.
fn negated_number(
    cursor: &mut Cursor,
    operator: UnaryOperator,
    start: usize,
) -> Result<Option<i64>, ParseError> {
    if operator != UnaryOperator::Negate {
        return Ok(None);
    }
    let Some(token) = cursor.eat(TokenKind::Integer) else {
        return Ok(None);
    };
    let negated = format!("-{}", token.span.text(cursor.source()));
    decode_integer(&negated, cursor.span_since(start)).map(Some)
}

fn eat_unary_operator(cursor: &mut Cursor) -> Option<UnaryOperator> {
    if cursor.eat_punct(Punct::Bang).is_some() {
        return Some(UnaryOperator::Not);
    }
    cursor
        .eat_punct(Punct::Minus)
        .map(|_| UnaryOperator::Negate)
}

/// A call, a field, or a `?`, applied left to right to whatever came before it.
fn postfix(cursor: &mut Cursor, records: RecordLiterals) -> Result<Expr, ParseError> {
    let start = cursor.offset();
    let mut expr = primary(cursor, records)?;
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
    comma_separated(cursor, Punct::RParen, |cursor| {
        expression(cursor, RecordLiterals::Allowed)
    })
}

fn primary(cursor: &mut Cursor, records: RecordLiterals) -> Result<Expr, ParseError> {
    let start = cursor.offset();
    let kind = primary_kind(cursor, records)?;
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
