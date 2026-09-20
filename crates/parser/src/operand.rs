//! One operand: a value, and the calls, fields, and `?` applied to it.
//!
//! An operator's operand lives here rather than in `expr`, so that the precedence chain reads
//! top down in one file and the recursion back into an expression crosses a module boundary.

use lumen_ast::{Arguments, Expr, ExprKind, NamedArgument, Path};
use lumen_lexer::{Keyword, Punct, TokenKind};

use crate::control::{if_expression, match_expression};
use crate::cursor::Cursor;
use crate::error::{Expected, ParseError, ParseErrorKind};
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

/// What a call passes, which is its values alone or each value with the parameter it is for.
///
/// The first argument settles which, because a call names all of its arguments or none of them.
/// Every argument after it is then held to the shape the first one chose, so `rename(old, to: new)`
/// and `rename(from: old, new)` are each refused here rather than carried on as something else.
/// `docs/specs/arguments.md` states the rule and `docs/specs/grammar.md` writes the two lists.
fn arguments(cursor: &mut Cursor) -> Result<Arguments, ParseError> {
    if names_them(cursor) {
        let written = comma_separated(cursor, Punct::RParen, Emptiness::Allowed, named_argument)?;
        return Ok(Arguments::Named(written));
    }
    let values = comma_separated(
        cursor,
        Punct::RParen,
        Emptiness::Allowed,
        positional_argument,
    )?;
    Ok(Arguments::Positional(values))
}

/// `from: old`: the parameter the value is passed for, and the value.
///
/// The first argument settled that this call names them, so one written without a name is the
/// half-named call of `docs/specs/arguments.md` rather than the start of an expression.
fn named_argument(cursor: &mut Cursor) -> Result<NamedArgument, ParseError> {
    if !names_them(cursor) {
        return Err(cursor.error_kind(ParseErrorKind::PartlyNamedCall));
    }
    let name = cursor.expect_name(Expected::Name)?;
    cursor.expect_punct(Punct::Colon)?;
    Ok(NamedArgument {
        name,
        value: expression(cursor, RecordLiterals::Allowed)?,
    })
}

/// `old`: the value alone, the first argument having settled that this call names none of them.
fn positional_argument(cursor: &mut Cursor) -> Result<Expr, ParseError> {
    if names_them(cursor) {
        return Err(cursor.error_kind(ParseErrorKind::PartlyNamedCall));
    }
    expression(cursor, RecordLiterals::Allowed)
}

/// Whether the call writes the parameter names, which the first argument says.
///
/// A `:` after a name opens a record field and nothing else, so a name followed by one here is
/// the parameter a value is passed for rather than the start of an expression.
fn names_them(cursor: &Cursor) -> bool {
    cursor.peek_kind(0) == Some(TokenKind::Identifier)
        && cursor.peek_kind(1) == Some(TokenKind::Punct(Punct::Colon))
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
        Some(TokenKind::Punct(Punct::LBracket)) => written_list(cursor),
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

/// `[first, second]`, or `[]`, which is the one way a list is written.
///
/// The elements are an ordinary comma-separated list, so one spans lines wherever a call's
/// arguments do, and a trailing comma is refused here as it is everywhere else.
fn written_list(cursor: &mut Cursor) -> Result<ExprKind, ParseError> {
    cursor.expect_punct(Punct::LBracket)?;
    let elements = comma_separated(cursor, Punct::RBracket, Emptiness::Allowed, |cursor| {
        expression(cursor, RecordLiterals::Allowed)
    })?;
    Ok(ExprKind::List(elements))
}

/// A name, or the record literal it heads, which is reached through a module or not.
fn name_or_record(cursor: &mut Cursor, records: RecordLiterals) -> Result<ExprKind, ParseError> {
    if records == RecordLiterals::Allowed && heads_a_record_of_a_module(cursor) {
        let base = cursor.expect_path(Expected::Expression)?;
        return record_literal(cursor, base);
    }
    let base = cursor.expect_name(Expected::Expression)?;
    if records == RecordLiterals::Forbidden || !cursor.at_punct(Punct::LBrace) {
        return Ok(ExprKind::Name(base));
    }
    record_literal(cursor, Path::bare(base))
}

/// `demo.User {`: a record of another module, which is the one dotted name a brace follows.
///
/// Every other dot opens a field or a call, which the postfix chain reads once the name is read.
fn heads_a_record_of_a_module(cursor: &Cursor) -> bool {
    cursor.peek_kind(1) == Some(TokenKind::Punct(Punct::Dot))
        && cursor.peek_kind(2) == Some(TokenKind::Identifier)
        && cursor.peek_kind(3) == Some(TokenKind::Punct(Punct::LBrace))
}
