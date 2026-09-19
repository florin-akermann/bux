//! Input the language has no use for still lexes; the lexer never fails.

use lumen_lexer::{Span, TokenKind, lex};

use crate::common::kinds;

#[test]
fn a_character_that_starts_no_token_is_unknown() {
    for text in ["@", "#", "$", "&", "~", "^", "`", "\\", ";"] {
        assert_eq!(kinds(text), [TokenKind::Unknown], "{text}");
    }
}

#[test]
fn an_unknown_character_is_one_token_and_lexing_continues_after_it() {
    let source = "x := 1 @ 2";
    assert_eq!(
        kinds(source),
        [
            TokenKind::Identifier,
            TokenKind::Punct(lumen_lexer::Punct::Walrus),
            TokenKind::Integer,
            TokenKind::Unknown,
            TokenKind::Integer
        ]
    );
    assert_eq!(lex(source)[3].span, Span::new(7, 1));
}

#[test]
fn a_run_of_unknown_characters_is_one_token_each() {
    assert_eq!(kinds("@@"), [TokenKind::Unknown, TokenKind::Unknown]);
}

#[test]
fn empty_input_lexes_to_no_tokens() {
    assert!(lex("").is_empty());
    assert!(lex("  \t\r").is_empty());
}
