//! The token cursor the whole parser reads through.
//!
//! The cursor is where the grammar's newline rule lives: comments never reach the parser, and
//! a newline reaches it only when the token before it could end a statement, a field, or an
//! arm. `docs/specs/grammar.md` states the rule; everything downstream then reads a token
//! stream in which every remaining newline terminates something.

use lumen_ast::{Name, Span};
use lumen_lexer::{Keyword, Punct, Token, TokenKind, lex};

use crate::error::{Expected, Found, ParseError, ParseErrorKind, token_error};

/// How deeply brackets may nest before the parser refuses the input.
///
/// Recursive descent recurses once per level of nesting, so without a budget a generated file
/// of twenty thousand nested parentheses overflows the stack and aborts the process instead of
/// reporting an error. An unoptimised build spends some eighteen kilobytes of stack per level,
/// and the smallest stack the toolchain runs on is the two megabytes a test thread gets, so
/// this budget leaves a threefold margin there. Canonical form never comes near it.
const MAX_NESTING: usize = 32;

pub(crate) struct Cursor<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    position: usize,
    depth: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: significant_tokens(source),
            position: 0,
            depth: 0,
        }
    }

    pub(crate) const fn source(&self) -> &'a str {
        self.source
    }

    /// Parses one level deeper, spending a level of the nesting budget for the duration.
    ///
    /// Every edge on which the grammar recurses goes through here, so the budget bounds how far
    /// the parser can descend however the nesting is written.
    pub(crate) fn nested<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T, ParseError>,
    ) -> Result<T, ParseError> {
        if self.depth == MAX_NESTING {
            return Err(self.error_kind(ParseErrorKind::NestingTooDeep));
        }
        self.depth += 1;
        let parsed = parse(self);
        self.depth -= 1;
        parsed
    }

    /// The identifier at the cursor, as a name.
    pub(crate) fn expect_name(&mut self, expected: Expected) -> Result<Name, ParseError> {
        let token = self.expect(TokenKind::Identifier, expected)?;
        Ok(Name {
            text: token.span.text(self.source).to_owned(),
            span: token.span,
        })
    }

    pub(crate) fn expect_punct(&mut self, punct: Punct) -> Result<Token, ParseError> {
        self.expect(TokenKind::Punct(punct), Expected::Punct(punct))
    }

    pub(crate) fn expect_keyword(&mut self, keyword: Keyword) -> Result<Token, ParseError> {
        self.expect(TokenKind::Keyword(keyword), Expected::Keyword(keyword))
    }

    /// Consumes the newline that ends an item or a statement; the end of the file ends one too.
    pub(crate) fn expect_end_of_line(&mut self) -> Result<(), ParseError> {
        if self.at_statement_end() {
            self.eat(TokenKind::Newline);
            return Ok(());
        }
        Err(self.error(Expected::EndOfLine))
    }

    /// Consumes a newline that terminates nothing, which is the one before a closing bracket.
    pub(crate) fn skip_newline(&mut self) {
        self.eat(TokenKind::Newline);
    }

    pub(crate) fn expect(
        &mut self,
        kind: TokenKind,
        expected: Expected,
    ) -> Result<Token, ParseError> {
        self.eat(kind).ok_or_else(|| self.error(expected))
    }

    pub(crate) fn eat_punct(&mut self, punct: Punct) -> Option<Token> {
        self.eat(TokenKind::Punct(punct))
    }

    pub(crate) fn eat_keyword(&mut self, keyword: Keyword) -> Option<Token> {
        self.eat(TokenKind::Keyword(keyword))
    }

    /// Whether nothing more of the current statement follows on this line.
    pub(crate) fn at_statement_end(&self) -> bool {
        self.at_end() || self.at(TokenKind::Newline) || self.at_punct(Punct::RBrace)
    }

    pub(crate) fn at_punct(&self, punct: Punct) -> bool {
        self.at(TokenKind::Punct(punct))
    }

    pub(crate) fn at_end(&self) -> bool {
        self.peek().is_none()
    }

    pub(crate) fn at(&self, kind: TokenKind) -> bool {
        self.peek().is_some_and(|token| token.kind == kind)
    }

    pub(crate) fn eat(&mut self, kind: TokenKind) -> Option<Token> {
        self.peek().filter(|token| token.kind == kind)?;
        self.advance()
    }

    pub(crate) fn advance(&mut self) -> Option<Token> {
        let token = self.peek()?;
        self.position += 1;
        Some(token)
    }

    /// The byte offset the next token starts at, which is where a node's span starts.
    pub(crate) fn offset(&self) -> usize {
        self.peek()
            .map_or_else(|| self.previous_end(), |token| token.span.start())
    }

    /// The span from `start` to the end of the last consumed token.
    pub(crate) fn span_since(&self, start: usize) -> Span {
        Span::new(start, self.previous_end().saturating_sub(start))
    }

    /// The error for finding what is at the cursor where `expected` was wanted.
    pub(crate) fn error(&self, expected: Expected) -> ParseError {
        match self.peek() {
            Some(token) => ParseError::new(token_error(expected, token, self.source), token.span),
            None => ParseError::new(
                ParseErrorKind::Expected {
                    expected,
                    found: Found::EndOfFile,
                },
                self.end_of_file_span(),
            ),
        }
    }

    /// The error for a failure that is not about which token was found.
    pub(crate) fn error_kind(&self, kind: ParseErrorKind) -> ParseError {
        let span = self
            .peek()
            .map_or_else(|| self.end_of_file_span(), |token| token.span);
        ParseError::new(kind, span)
    }

    /// The kind `distance` tokens ahead, for the places the grammar needs a second look.
    pub(crate) fn peek_kind(&self, distance: usize) -> Option<TokenKind> {
        self.tokens
            .get(self.position + distance)
            .map(|token| token.kind)
    }

    pub(crate) fn peek(&self) -> Option<Token> {
        self.tokens.get(self.position).copied()
    }

    /// Where the last consumed token ended, which is where an adjacent token would start.
    pub(crate) fn previous_end(&self) -> usize {
        self.position
            .checked_sub(1)
            .map_or(0, |index| self.tokens[index].span.end())
    }

    /// The last token, which is what an error at the end of the file points at.
    fn end_of_file_span(&self) -> Span {
        self.tokens
            .last()
            .map_or_else(|| Span::new(0, self.source.len()), |token| token.span)
    }
}

/// The tokens the grammar sees: no comments, and only the newlines that terminate something.
fn significant_tokens(source: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    for token in lex(source) {
        let keep = match token.kind {
            TokenKind::LineComment => false,
            TokenKind::Newline => tokens
                .last()
                .is_some_and(|previous| ends_a_line(previous.kind)),
            _ => true,
        };
        if keep {
            tokens.push(token);
        }
    }
    tokens
}

/// Whether a token can be the last one of a statement, a field, a variant, or a match arm.
fn ends_a_line(kind: TokenKind) -> bool {
    match kind {
        TokenKind::Identifier | TokenKind::Integer | TokenKind::String => true,
        TokenKind::Keyword(keyword) => matches!(
            keyword,
            Keyword::True | Keyword::False | Keyword::Break | Keyword::Continue | Keyword::Return
        ),
        TokenKind::Punct(punct) => matches!(
            punct,
            Punct::RParen | Punct::RBracket | Punct::RBrace | Punct::Gt | Punct::Question
        ),
        _ => false,
    }
}
