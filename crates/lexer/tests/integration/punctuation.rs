//! Operators and punctuation, and the longest-match rule between them.

use lumen_lexer::{Punct, TokenKind};

use crate::common::kinds;

#[test]
fn each_punctuation_lexes_to_its_kind() {
    let punctuation = [
        (":=", Punct::Walrus),
        ("==", Punct::EqEq),
        ("!=", Punct::BangEq),
        ("<=", Punct::LtEq),
        (">=", Punct::GtEq),
        ("+=", Punct::PlusEq),
        ("&&", Punct::AndAnd),
        ("||", Punct::OrOr),
        ("->", Punct::Arrow),
        ("=>", Punct::FatArrow),
        ("=", Punct::Eq),
        ("<", Punct::Lt),
        (">", Punct::Gt),
        ("+", Punct::Plus),
        ("-", Punct::Minus),
        ("*", Punct::Star),
        ("/", Punct::Slash),
        ("%", Punct::Percent),
        ("!", Punct::Bang),
        ("?", Punct::Question),
        (".", Punct::Dot),
        (",", Punct::Comma),
        (":", Punct::Colon),
        ("|", Punct::Pipe),
        ("(", Punct::LParen),
        (")", Punct::RParen),
        ("{", Punct::LBrace),
        ("}", Punct::RBrace),
        ("[", Punct::LBracket),
        ("]", Punct::RBracket),
    ];
    for (text, punct) in punctuation {
        assert_eq!(kinds(text), [TokenKind::Punct(punct)], "{text}");
    }
}

#[test]
fn the_longest_punctuation_wins() {
    assert_eq!(kinds(":="), [TokenKind::Punct(Punct::Walrus)]);
    assert_eq!(kinds("=="), [TokenKind::Punct(Punct::EqEq)]);
    assert_eq!(kinds("=>"), [TokenKind::Punct(Punct::FatArrow)]);
    assert_eq!(kinds("->"), [TokenKind::Punct(Punct::Arrow)]);
    assert_eq!(
        kinds("==="),
        [TokenKind::Punct(Punct::EqEq), TokenKind::Punct(Punct::Eq)]
    );
}

#[test]
fn punctuation_separated_by_a_blank_is_two_tokens() {
    assert_eq!(
        kinds("- >"),
        [TokenKind::Punct(Punct::Minus), TokenKind::Punct(Punct::Gt)]
    );
}

#[test]
fn adjacent_punctuation_that_forms_no_longer_token_stays_apart() {
    assert_eq!(
        kinds(")("),
        [
            TokenKind::Punct(Punct::RParen),
            TokenKind::Punct(Punct::LParen)
        ]
    );
    assert_eq!(
        kinds("a.b"),
        [
            TokenKind::Identifier,
            TokenKind::Punct(Punct::Dot),
            TokenKind::Identifier
        ]
    );
}

#[test]
fn the_text_of_a_punctuation_lexes_back_to_it() {
    for punct in ALL_PUNCTUATION {
        assert_eq!(
            kinds(punct.text()),
            [TokenKind::Punct(punct)],
            "{:?} spells {:?}",
            punct,
            punct.text()
        );
    }
}

/// Every punctuation of the version 0.1 surface, in the order `docs/specs/lexer.md` lists them.
const ALL_PUNCTUATION: [Punct; 30] = [
    Punct::Walrus,
    Punct::EqEq,
    Punct::BangEq,
    Punct::LtEq,
    Punct::GtEq,
    Punct::PlusEq,
    Punct::AndAnd,
    Punct::OrOr,
    Punct::Arrow,
    Punct::FatArrow,
    Punct::Eq,
    Punct::Lt,
    Punct::Gt,
    Punct::Plus,
    Punct::Minus,
    Punct::Star,
    Punct::Slash,
    Punct::Percent,
    Punct::Bang,
    Punct::Question,
    Punct::Dot,
    Punct::Comma,
    Punct::Colon,
    Punct::Pipe,
    Punct::LParen,
    Punct::RParen,
    Punct::LBrace,
    Punct::RBrace,
    Punct::LBracket,
    Punct::RBracket,
];
