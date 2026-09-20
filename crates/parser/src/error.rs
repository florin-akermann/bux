//! The one error a parse can fail with.

use std::fmt;

use lumen_diagnostics::{Code, Diagnostic};
use lumen_lexer::{Keyword, Punct, Span, Token, TokenKind};

/// Why a parse failed, and where.
///
/// Parsing stops at the first error, so there is exactly one of these per failed parse. The
/// wording is specified in `docs/specs/grammar.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    span: Span,
}

impl ParseError {
    pub(crate) const fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// The source the error points at.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// This failure as the diagnostic the reader is shown.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new(
            self.kind.code(),
            self.message(),
            self.span,
            self.help().map(str::to_owned),
        )
    }

    /// The `error:` line, without its prefix.
    #[must_use]
    pub fn message(&self) -> String {
        self.kind.to_string()
    }

    /// The `help:` line, when there is a fix worth naming.
    #[must_use]
    pub const fn help(&self) -> Option<&'static str> {
        self.kind.help()
    }
}

/// What went wrong, in the words the reader sees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ParseErrorKind {
    Expected { expected: Expected, found: Found },
    UnterminatedString,
    UnknownCharacter,
    IntegerTooLarge,
    UnknownEscape(char),
    ChainedComparison,
    NestingTooDeep,
    AssignedToValue,
    PartlyNamedCall,
}

impl ParseErrorKind {
    /// The code this failure is refused with, which `docs/specs/diagnostics.md` lists.
    const fn code(&self) -> Code {
        match self {
            Self::Expected { .. } => Code::UnexpectedToken,
            Self::UnterminatedString => Code::UnterminatedString,
            Self::UnknownCharacter => Code::UnknownCharacter,
            Self::IntegerTooLarge => Code::IntegerTooLarge,
            Self::UnknownEscape(_) => Code::UnknownEscape,
            Self::ChainedComparison => Code::ChainedComparison,
            Self::NestingTooDeep => Code::NestingTooDeep,
            Self::AssignedToValue => Code::AssignedToValue,
            Self::PartlyNamedCall => Code::PartlyNamedCall,
        }
    }

    const fn help(&self) -> Option<&'static str> {
        match self {
            Self::Expected {
                expected: Expected::FunctionName,
                ..
            } => Some("every function has a name; Lumen has no anonymous functions"),
            Self::UnterminatedString => Some("add a closing `\"` before the end of the line"),
            Self::IntegerTooLarge => Some("the largest whole number is 9223372036854775807"),
            Self::UnknownEscape(_) => {
                Some("the escapes are `\\\"`, `\\\\`, `\\n`, `\\t`, and `\\r`")
            }
            Self::ChainedComparison => Some("compare twice and join the two with `&&`"),
            Self::NestingTooDeep => Some("brackets nest at most 32 deep; name a part of it"),
            Self::AssignedToValue => Some("build the value it becomes: `user { name: \"Bob\" }`"),
            Self::PartlyNamedCall => Some("a call names all of its arguments or none of them"),
            _ => None,
        }
    }
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expected { expected, found } => write!(f, "expected {expected}, found {found}"),
            Self::UnterminatedString => write!(f, "this string has no closing quote"),
            Self::UnknownCharacter => write!(f, "this character is not part of the language"),
            Self::IntegerTooLarge => write!(f, "this number does not fit in a whole number"),
            Self::UnknownEscape(escaped) => write!(f, "`\\{escaped}` is not an escape"),
            Self::ChainedComparison => write!(f, "comparisons do not chain"),
            Self::NestingTooDeep => write!(f, "this nests too deeply to parse"),
            Self::AssignedToValue => write!(f, "only a name is assigned to"),
            Self::PartlyNamedCall => {
                write!(f, "this call names some of its arguments and not others")
            }
        }
    }
}

/// What the parser was looking for, named as a thing the reader writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Expected {
    Punct(Punct),
    Keyword(Keyword),
    Name,
    FunctionName,
    ExternKind,
    JavaName,
    Trait,
    Item,
    Expression,
    Type,
    Pattern,
    EndOfLine,
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Punct(punct) => write!(f, "`{}`", punct.text()),
            Self::Keyword(keyword) => write!(f, "`{}`", keyword.text()),
            Self::Name => write!(f, "a name"),
            Self::FunctionName => write!(f, "a function name"),
            Self::ExternKind => write!(f, "`field`, `static`, `method`, or `new`"),
            Self::JavaName => write!(f, "a Java name in quotes"),
            Self::Trait => write!(f, "a trait"),
            Self::Item => write!(
                f,
                "an import, a type, a trait, an instance, a derive, a function, or an extern"
            ),
            Self::Expression => write!(f, "an expression"),
            Self::Type => write!(f, "a type"),
            Self::Pattern => write!(f, "a pattern"),
            Self::EndOfLine => write!(f, "the end of the line"),
        }
    }
}

/// What was there instead, named the same way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Found {
    Word(String),
    Punct(Punct),
    Number,
    Text,
    EndOfLine,
    EndOfFile,
}

/// What kind of failure `token` is: it may be the error itself, or merely not what was wanted.
pub(crate) fn token_error(expected: Expected, token: Token, source: &str) -> ParseErrorKind {
    let found = match token.kind {
        TokenKind::Unknown => return ParseErrorKind::UnknownCharacter,
        TokenKind::UnterminatedString => return ParseErrorKind::UnterminatedString,
        TokenKind::Identifier => Found::Word(token.span.text(source).to_owned()),
        TokenKind::Keyword(keyword) => Found::Word(keyword.text().to_owned()),
        TokenKind::Punct(punct) => Found::Punct(punct),
        TokenKind::Integer => Found::Number,
        TokenKind::String => Found::Text,
        TokenKind::Newline | TokenKind::LineComment => Found::EndOfLine,
    };
    ParseErrorKind::Expected { expected, found }
}

impl fmt::Display for Found {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Word(word) => write!(f, "`{word}`"),
            Self::Punct(punct) => write!(f, "`{}`", punct.text()),
            Self::Number => write!(f, "a number"),
            Self::Text => write!(f, "a string"),
            Self::EndOfLine => write!(f, "the end of the line"),
            Self::EndOfFile => write!(f, "the end of the file"),
        }
    }
}
