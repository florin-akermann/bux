//! Patterns, as written in the arms of a `match`.

use lumen_ast::{Pattern, PatternKind};
use lumen_lexer::{Punct, TokenKind};

use crate::cursor::Cursor;
use crate::error::{Expected, ParseError};
use crate::list::comma_separated;
use crate::literal::{Literal, eat_literal};

pub(crate) fn pattern(cursor: &mut Cursor) -> Result<Pattern, ParseError> {
    let start = cursor.offset();
    let kind = pattern_kind(cursor)?;
    Ok(Pattern {
        kind,
        span: cursor.span_since(start),
    })
}

fn pattern_kind(cursor: &mut Cursor) -> Result<PatternKind, ParseError> {
    if let Some(literal) = eat_literal(cursor)? {
        return Ok(of_literal(literal));
    }
    if cursor.peek_kind(0) == Some(TokenKind::Identifier) {
        return named(cursor);
    }
    Err(cursor.error(Expected::Pattern))
}

fn of_literal(literal: Literal) -> PatternKind {
    match literal {
        Literal::Integer(value) => PatternKind::Integer(value),
        Literal::String(value) => PatternKind::String(value),
        Literal::Bool(value) => PatternKind::Bool(value),
    }
}

/// A bare name, or a variant matched on what it carries.
fn named(cursor: &mut Cursor) -> Result<PatternKind, ParseError> {
    let name = cursor.expect_name(Expected::Pattern)?;
    if cursor.eat_punct(Punct::LParen).is_some() {
        if cursor.at_punct(Punct::RParen) {
            return Err(cursor.error(Expected::Pattern));
        }
        let elements = comma_separated(cursor, Punct::RParen, pattern)?;
        return Ok(PatternKind::Tuple { name, elements });
    }
    if cursor.eat_punct(Punct::LBrace).is_some() {
        let fields = comma_separated(cursor, Punct::RBrace, |cursor| {
            cursor.expect_name(Expected::Name)
        })?;
        return Ok(PatternKind::Record { name, fields });
    }
    Ok(PatternKind::Name(name))
}
