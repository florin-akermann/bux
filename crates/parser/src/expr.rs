//! Expressions, by precedence.

use lumen_ast::{BinaryOperator, Expr, ExprKind, UnaryOperator};
use lumen_lexer::{Punct, TokenKind};

use crate::cursor::Cursor;
use crate::error::{ParseError, ParseErrorKind};
use crate::literal::decode_integer;
use crate::operand::{postfix, postfix_chain};

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
    if let Some(value) = negated_number(cursor, operator, start)? {
        let number = Expr {
            kind: ExprKind::Integer(value),
            span: cursor.span_since(start),
        };
        return postfix_chain(cursor, number, start);
    }
    let operand = cursor.nested(|cursor| unary(cursor, records))?;
    Ok(Expr {
        kind: ExprKind::Unary {
            operator,
            operand: Box::new(operand),
        },
        span: cursor.span_since(start),
    })
}

/// A `-` before a number is part of that number, so the smallest whole number can be written.
///
/// Whatever blanks sit between the two, `- 5` and `-5` are the same number, which is what lets
/// the formatter write the one canonical spelling of it without changing what the source says.
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
