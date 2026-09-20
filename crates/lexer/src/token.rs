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

/// Declares a fixed vocabulary: the kinds, the text that spells each, and the list of them all.
///
/// Writing those three out separately is what lets them drift, and the drift that matters is a
/// spelling the lexer scans for but no kind names, or a kind nothing scans for. Here a kind
/// cannot be declared without its text, nor without joining the list the scanner searches.
macro_rules! vocabulary {
    (
        $(#[$about:meta])*
        $name:ident, $every:ident, $order:literal:
        $($(#[$each:meta])* $kind:ident => $text:literal,)+
    ) => {
        $(#[$about])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $($(#[$each])* $kind,)+
        }

        impl $name {
            /// The text that spells this one.
            #[must_use]
            pub const fn text(self) -> &'static str {
                match self {
                    $(Self::$kind => $text,)+
                }
            }
        }

        #[doc = $order]
        pub(crate) const $every: &[$name] = &[$($name::$kind,)+];
    };
}

vocabulary! {
    /// The words the grammar reserves.
    ///
    /// `_` is reserved among them although it is no word: it spells the discard of
    /// `docs/specs/discarding.md` and is never an identifier, while `_x` is a name as ever.
    Keyword, KEYWORDS, "Every keyword, so that a word can be looked up among them.":
    Fn => "fn",
    Type => "type",
    Trait => "trait",
    Instance => "instance",
    Var => "var",
    If => "if",
    Else => "else",
    For => "for",
    In => "in",
    Match => "match",
    Break => "break",
    Continue => "continue",
    Return => "return",
    Import => "import",
    True => "true",
    False => "false",
    /// `_`, which is written on the left of `=` and nowhere else.
    Underscore => "_",
}

vocabulary! {
    /// Operators and punctuation.
    Punct, PUNCTUATION, "Every punctuation, longest first so `:=` wins over `:` and `->` over `-`.":
    Walrus => ":=",
    EqEq => "==",
    BangEq => "!=",
    LtEq => "<=",
    GtEq => ">=",
    PlusEq => "+=",
    AndAnd => "&&",
    OrOr => "||",
    Arrow => "->",
    FatArrow => "=>",
    Eq => "=",
    Lt => "<",
    Gt => ">",
    Plus => "+",
    Minus => "-",
    Star => "*",
    Slash => "/",
    Percent => "%",
    Bang => "!",
    Question => "?",
    Dot => ".",
    Comma => ",",
    Colon => ":",
    Pipe => "|",
    LParen => "(",
    RParen => ")",
    LBrace => "{",
    RBrace => "}",
    LBracket => "[",
    RBracket => "]",
}
