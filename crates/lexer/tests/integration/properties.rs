//! The invariants of `docs/specs/lexer.md`, checked on arbitrary text.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_lexer::{Token, TokenKind, lex};

use crate::common::kinds;

fn is_blank(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r')
}

fn source_text() -> impl hegel::Generator<String> {
    gs::text()
}

#[hegel::test]
fn lexing_never_panics_and_is_deterministic(tc: TestCase) {
    let source = tc.draw(source_text());
    assert_eq!(lex(&source), lex(&source));
}

#[hegel::test]
fn spans_are_ordered_non_overlapping_non_empty_and_on_character_boundaries(tc: TestCase) {
    let source = tc.draw(source_text());
    let mut previous_end = 0;
    for Token { span, .. } in lex(&source) {
        assert!(
            span.start() >= previous_end,
            "{span:?} overlaps or precedes the token before it"
        );
        assert!(span.start() < span.end(), "{span:?} is empty");
        assert!(span.end() <= source.len(), "{span:?} runs past the input");
        assert!(source.is_char_boundary(span.start()) && source.is_char_boundary(span.end()));
        previous_end = span.end();
    }
}

#[hegel::test]
fn every_byte_outside_a_span_is_a_blank_and_no_span_holds_a_carriage_return(tc: TestCase) {
    let source = tc.draw(source_text());
    let mut covered = vec![false; source.len()];
    for Token { span, .. } in lex(&source) {
        assert!(
            !span.text(&source).contains('\r'),
            "{span:?} holds a carriage return"
        );
        covered[span.range()].fill(true);
    }
    for (index, byte) in source.bytes().enumerate() {
        assert!(
            covered[index] || is_blank(byte),
            "byte {index} ({byte:?}) is neither lexed nor blank"
        );
    }
}

/// Every token class but comments and newlines, which a single space could not keep apart.
const SPACE_SEPARABLE_TOKENS: [(&str, TokenKind); 10] = [
    ("user", TokenKind::Identifier),
    ("x1", TokenKind::Identifier),
    ("fn", TokenKind::Keyword(lumen_lexer::Keyword::Fn)),
    ("match", TokenKind::Keyword(lumen_lexer::Keyword::Match)),
    ("42", TokenKind::Integer),
    (r#""a b""#, TokenKind::String),
    (r#""\"""#, TokenKind::String),
    (":=", TokenKind::Punct(lumen_lexer::Punct::Walrus)),
    ("->", TokenKind::Punct(lumen_lexer::Punct::Arrow)),
    ("@", TokenKind::Unknown),
];

#[hegel::test]
fn token_texts_joined_by_spaces_lex_back_to_their_kinds(tc: TestCase) {
    let tokens: Vec<(&str, TokenKind)> =
        tc.draw(gs::vecs(gs::sampled_from(&SPACE_SEPARABLE_TOKENS)));
    let source = tokens
        .iter()
        .map(|(text, _)| *text)
        .collect::<Vec<_>>()
        .join(" ");
    let expected: Vec<TokenKind> = tokens.iter().map(|(_, kind)| *kind).collect();
    assert_eq!(kinds(&source), expected);
}
