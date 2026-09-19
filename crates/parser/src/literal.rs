//! Literals: the one place an expression and a pattern spell a value the same way.

use lumen_ast::Span;
use lumen_lexer::{Keyword, TokenKind};

use crate::cursor::Cursor;
use crate::error::{ParseError, ParseErrorKind};

/// A value written out in source, which an expression and a pattern each make their own node of.
pub(crate) enum Literal {
    Integer(i64),
    String(String),
    Bool(bool),
}

/// The literal at the cursor, when what is there is one.
pub(crate) fn eat_literal(cursor: &mut Cursor) -> Result<Option<Literal>, ParseError> {
    let Some(token) = cursor.peek() else {
        return Ok(None);
    };
    let text = token.span.text(cursor.source());
    let literal = match token.kind {
        TokenKind::Integer => Literal::Integer(decode_integer(text, token.span)?),
        TokenKind::String => Literal::String(decode_string(text, token.span)?),
        TokenKind::Keyword(Keyword::True) => Literal::Bool(true),
        TokenKind::Keyword(Keyword::False) => Literal::Bool(false),
        _ => return Ok(None),
    };
    cursor.advance();
    Ok(Some(literal))
}

/// The value of an integer literal, whose text the lexer has already checked is all digits.
pub(crate) fn decode_integer(text: &str, span: Span) -> Result<i64, ParseError> {
    text.parse()
        .map_err(|_| ParseError::new(ParseErrorKind::IntegerTooLarge, span))
}

/// The value of a string literal, with its escapes resolved.
pub(crate) fn decode_string(text: &str, span: Span) -> Result<String, ParseError> {
    let body = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("a string token opens and closes with a quote");
    let mut value = String::new();
    let mut characters = body.char_indices();
    while let Some((index, character)) = characters.next() {
        if character != '\\' {
            value.push(character);
            continue;
        }
        let (_, escaped) = characters
            .next()
            .expect("a string token never ends on a backslash");
        value.push(unescape(escaped).ok_or_else(|| unknown_escape(escaped, span, index))?);
    }
    Ok(value)
}

/// What a backslash makes of the character after it, when the language knows the escape.
fn unescape(escaped: char) -> Option<char> {
    match escaped {
        '"' => Some('"'),
        '\\' => Some('\\'),
        'n' => Some('\n'),
        't' => Some('\t'),
        'r' => Some('\r'),
        _ => None,
    }
}

/// The error points at the backslash and the character it failed to escape.
fn unknown_escape(escaped: char, literal: Span, index_in_body: usize) -> ParseError {
    let start = literal.start() + '"'.len_utf8() + index_in_body;
    let span = Span::new(start, '\\'.len_utf8() + escaped.len_utf8());
    ParseError::new(ParseErrorKind::UnknownEscape(escaped), span)
}
