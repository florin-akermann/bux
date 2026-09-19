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

impl Keyword {
    /// The word that spells this keyword.
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::Fn => "fn",
            Self::Type => "type",
            Self::Var => "var",
            Self::If => "if",
            Self::Else => "else",
            Self::For => "for",
            Self::In => "in",
            Self::Match => "match",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Return => "return",
            Self::Import => "import",
            Self::True => "true",
            Self::False => "false",
        }
    }
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

impl Punct {
    /// The characters that spell this punctuation.
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::Walrus => ":=",
            Self::EqEq => "==",
            Self::BangEq => "!=",
            Self::LtEq => "<=",
            Self::GtEq => ">=",
            Self::PlusEq => "+=",
            Self::AndAnd => "&&",
            Self::OrOr => "||",
            Self::Arrow => "->",
            Self::FatArrow => "=>",
            Self::Eq => "=",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Bang => "!",
            Self::Question => "?",
            Self::Dot => ".",
            Self::Comma => ",",
            Self::Colon => ":",
            Self::Pipe => "|",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::LBracket => "[",
            Self::RBracket => "]",
        }
    }
}

/// Every keyword, so that a word can be looked up among them.
pub(crate) const KEYWORDS: [Keyword; 14] = [
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
];

/// Every punctuation, longest first so that `:=` wins over `:` and `->` over `-`.
pub(crate) const PUNCTUATION: [Punct; 30] = [
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
