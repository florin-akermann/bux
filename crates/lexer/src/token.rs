//! Tokens: a kind and the span of source text it covers.

use crate::Span;

/// One token of the source: what it is, and where its text lies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// The token classes of the version 0.1 surface, plus the two that keep lexing total.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Keyword(Keyword),
    Integer,
    String,
    Punct(Punct),
    Newline,
    LineComment,
    /// A `"` whose closing `"` does not arrive before a newline or the end of input.
    UnterminatedString,
    /// One character the language has no use for; the parser reports it.
    Unknown,
}

/// The words the version 0.1 grammar reserves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keyword {
    Fn,
    Type,
    Var,
    If,
    Else,
    For,
    In,
    Match,
    Break,
    Continue,
    Return,
    Import,
    True,
    False,
}

/// Operators and punctuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Punct {
    /// `:=`
    Walrus,
    /// `==`
    EqEq,
    /// `!=`
    BangEq,
    /// `<=`
    LtEq,
    /// `>=`
    GtEq,
    /// `+=`
    PlusEq,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `->`
    Arrow,
    /// `=>`
    FatArrow,
    /// `=`
    Eq,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `!`
    Bang,
    /// `?`
    Question,
    /// `.`
    Dot,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `|`
    Pipe,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
}

/// Each keyword beside the word that spells it.
pub(crate) const KEYWORDS: [(&str, Keyword); 14] = [
    ("fn", Keyword::Fn),
    ("type", Keyword::Type),
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

/// Each punctuation beside its text, longest first so that `:=` wins over `:`.
pub(crate) const PUNCTUATION: [(&str, Punct); 30] = [
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
