//! Patterns, as written in the arms of a `match`.

use lumen_ast::{Pattern, PatternKind};
use lumen_lexer::{Keyword, Punct, TokenKind};

use crate::cursor::Cursor;
use crate::error::{Expected, ParseError};
use crate::list::{Emptiness, comma_separated};
use crate::literal::{Literal, eat_literal};

/// A pattern, which is one alternative or several written with a `|` between them.
pub(crate) fn pattern(cursor: &mut Cursor) -> Result<Pattern, ParseError> {
    let start = cursor.offset();
    let first = alternative(cursor)?;
    if !cursor.at_punct(Punct::Pipe) {
        return Ok(first);
    }
    let mut alternatives = vec![first];
    while cursor.eat_punct(Punct::Pipe).is_some() {
        alternatives.push(alternative(cursor)?);
    }
    Ok(Pattern {
        kind: PatternKind::Or(alternatives),
        span: cursor.span_since(start),
    })
}

/// One alternative, which is every form of pattern but the `|` that joins them.
fn alternative(cursor: &mut Cursor) -> Result<Pattern, ParseError> {
    let start = cursor.offset();
    let kind = pattern_kind(cursor)?;
    Ok(Pattern {
        kind,
        span: cursor.span_since(start),
    })
}

fn pattern_kind(cursor: &mut Cursor) -> Result<PatternKind, ParseError> {
    if cursor.eat_keyword(Keyword::Underscore).is_some() {
        return Ok(PatternKind::Wildcard);
    }
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

/// A name, reached through a module or not, or a variant matched on what it carries.
fn named(cursor: &mut Cursor) -> Result<PatternKind, ParseError> {
    let path = cursor.expect_path(Expected::Pattern)?;
    if cursor.eat_punct(Punct::LParen).is_some() {
        let elements = cursor.nested(|cursor| {
            comma_separated(cursor, Punct::RParen, Emptiness::Forbidden, pattern)
        })?;
        return Ok(PatternKind::Tuple { path, elements });
    }
    if cursor.eat_punct(Punct::LBrace).is_some() {
        let fields = comma_separated(cursor, Punct::RBrace, Emptiness::Forbidden, |cursor| {
            cursor.expect_name(Expected::Name)
        })?;
        return Ok(PatternKind::Record { path, fields });
    }
    Ok(PatternKind::Name(path))
}
