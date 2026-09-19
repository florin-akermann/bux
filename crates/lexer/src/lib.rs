//! The Lumen lexer: source text to tokens with byte spans.
//!
//! Lexing is total. Every input lexes to a token sequence; text the language has no use for
//! becomes an [`TokenKind::Unknown`] or [`TokenKind::UnterminatedString`] token, and the parser
//! is the phase that reports it. The behaviour is specified in `docs/specs/lexer.md`.

mod span;
mod token;

pub use span::Span;
pub use token::{Keyword, Punct, Token, TokenKind};

use token::{KEYWORDS, PUNCTUATION};

/// Lexes `source` into its tokens, in source order.
///
/// Blanks (spaces, tabs, carriage returns) separate tokens and produce none; every other byte
/// of `source` lies inside exactly one token's span.
#[must_use]
pub fn lex(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut cursor = prefix_len(source, is_blank);
    while cursor < source.len() {
        let rest = &source[cursor..];
        let (kind, len) = scan_token(rest);
        tokens.push(Token {
            kind,
            span: Span::new(cursor, len),
        });
        cursor += len + prefix_len(&rest[len..], is_blank);
    }
    tokens
}

/// The kind and byte length of the token at the start of `rest`, which is non-empty and does
/// not start with a blank.
fn scan_token(rest: &str) -> (TokenKind, usize) {
    let first = rest
        .chars()
        .next()
        .expect("scan_token is given non-empty text");
    if first == '\n' {
        (TokenKind::Newline, 1)
    } else if rest.starts_with("//") {
        (
            TokenKind::LineComment,
            prefix_len(rest, |c| !is_line_end(c)),
        )
    } else if first.is_ascii_digit() {
        (TokenKind::Integer, prefix_len(rest, |c| c.is_ascii_digit()))
    } else if is_identifier_start(first) {
        scan_word(rest)
    } else if first == '"' {
        scan_string(rest)
    } else if let Some((text, punct)) = PUNCTUATION.iter().find(|(text, _)| rest.starts_with(text))
    {
        (TokenKind::Punct(*punct), text.len())
    } else {
        (TokenKind::Unknown, first.len_utf8())
    }
}

/// An identifier, or the keyword it spells.
fn scan_word(rest: &str) -> (TokenKind, usize) {
    let len = prefix_len(rest, is_identifier_continue);
    let word = &rest[..len];
    let kind = KEYWORDS
        .iter()
        .find(|(text, _)| *text == word)
        .map_or(TokenKind::Identifier, |(_, keyword)| {
            TokenKind::Keyword(*keyword)
        });
    (kind, len)
}

/// A string from its opening quote to the first unescaped `"` on the same line, or an
/// unterminated string reaching the end of the line or of the input.
fn scan_string(rest: &str) -> (TokenKind, usize) {
    let mut chars = rest.char_indices().skip(1).peekable();
    while let Some((index, c)) = chars.next() {
        match c {
            '"' => return (TokenKind::String, index + 1),
            c if is_line_end(c) => return (TokenKind::UnterminatedString, index),
            '\\' => {
                chars.next_if(|(_, escaped)| !is_line_end(*escaped));
            }
            _ => {}
        }
    }
    (TokenKind::UnterminatedString, rest.len())
}

/// The byte length of the longest prefix of `text` whose characters all satisfy `keep`.
fn prefix_len(text: &str, keep: impl Fn(char) -> bool) -> usize {
    text.find(|c| !keep(c)).unwrap_or(text.len())
}

fn is_blank(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r')
}

/// A carriage return ends a line too, so that CRLF input never puts a `\r` inside a token.
fn is_line_end(c: char) -> bool {
    matches!(c, '\n' | '\r')
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_identifier_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
