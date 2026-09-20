//! Words: identifiers, and the keywords the grammar reserves.

use lumen_lexer::{Keyword, Span, TokenKind, lex};

use crate::common::{kinds, texts};

#[test]
fn a_word_is_an_identifier() {
    assert_eq!(kinds("find_user"), [TokenKind::Identifier]);
    assert_eq!(texts("find_user"), ["find_user"]);
}

#[test]
fn an_identifier_is_the_longest_run_of_identifier_characters() {
    assert_eq!(texts("x1 y_2 _z"), ["x1", "y_2", "_z"]);
    assert_eq!(kinds("x1 y_2 _z"), [TokenKind::Identifier; 3]);
}

#[test]
fn a_reserved_word_is_its_keyword() {
    let reserved = [
        ("fn", Keyword::Fn),
        ("type", Keyword::Type),
        ("trait", Keyword::Trait),
        ("instance", Keyword::Instance),
        ("derive", Keyword::Derive),
        ("var", Keyword::Var),
        ("if", Keyword::If),
        ("else", Keyword::Else),
        ("for", Keyword::For),
        ("in", Keyword::In),
        ("match", Keyword::Match),
        ("break", Keyword::Break),
        ("continue", Keyword::Continue),
        ("return", Keyword::Return),
        ("import", Keyword::Import),
        ("true", Keyword::True),
        ("false", Keyword::False),
    ];
    for (word, keyword) in reserved {
        assert_eq!(kinds(word), [TokenKind::Keyword(keyword)], "{word}");
    }
}

#[test]
fn a_word_that_merely_starts_with_a_keyword_is_an_identifier() {
    assert_eq!(kinds("fnord"), [TokenKind::Identifier]);
    assert_eq!(kinds("format"), [TokenKind::Identifier]);
    assert_eq!(kinds("derived"), [TokenKind::Identifier]);
}

#[test]
fn the_type_names_of_the_prelude_are_ordinary_identifiers() {
    assert_eq!(
        kinds("Option Result Int String"),
        [TokenKind::Identifier; 4]
    );
}

#[test]
fn a_non_ascii_letter_is_an_unknown_token_covering_the_whole_character() {
    let tokens = lex("é");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Unknown);
    assert_eq!(tokens[0].span, Span::new(0, 2));
}

#[test]
fn a_bare_underscore_is_the_discard_and_never_an_identifier() {
    assert_eq!(kinds("_"), [TokenKind::Keyword(Keyword::Underscore)]);
}

#[test]
fn an_underscore_with_a_word_on_it_is_an_ordinary_identifier() {
    assert_eq!(kinds("_x"), [TokenKind::Identifier]);
    assert_eq!(kinds("__"), [TokenKind::Identifier]);
    assert_eq!(kinds("total_paid"), [TokenKind::Identifier]);
}

#[test]
fn the_word_of_a_keyword_lexes_back_to_it() {
    for keyword in ALL_KEYWORDS {
        assert_eq!(
            kinds(keyword.text()),
            [TokenKind::Keyword(keyword)],
            "{:?} spells {:?}",
            keyword,
            keyword.text()
        );
    }
}

/// Every keyword of the version 0.1 surface, in the order `docs/specs/lexer.md` lists them.
const ALL_KEYWORDS: [Keyword; 15] = [
    Keyword::Fn,
    Keyword::Type,
    Keyword::Var,
    Keyword::If,
    Keyword::Else,
    Keyword::For,
    Keyword::In,
    Keyword::Match,
    Keyword::Break,
    Keyword::Continue,
    Keyword::Return,
    Keyword::Import,
    Keyword::True,
    Keyword::False,
    Keyword::Underscore,
];
