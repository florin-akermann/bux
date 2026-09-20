//! The buffer canonical text is written into, and the comments woven back into it.
//!
//! `docs/specs/formatting.md` states the layout; this type holds the two pieces of state that
//! layout needs, the current indentation and the comments not yet written.

use lumen_ast::{Path, Span};
use lumen_lexer::{TokenKind, lex};

/// One level of indentation, as `docs/specs/formatting.md` fixes it.
const INDENT: &str = "    ";

pub(crate) struct Printer<'a> {
    source: &'a str,
    /// The line comments of the source, in source order.
    comments: Vec<Span>,
    /// How many of them have been written.
    written: usize,
    text: String,
    depth: usize,
}

impl<'a> Printer<'a> {
    /// A printer for text that carries no comment, which is the only thing a source is read for.
    pub(crate) fn without_comments() -> Self {
        Self::new("")
    }

    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            comments: comments_of(source),
            written: 0,
            text: String::new(),
            depth: 0,
        }
    }

    /// Writes the comments that belong above the line `span` begins on.
    ///
    /// A comment written at the end of a line therefore moves to its own line above that line,
    /// and a comment already alone on its line stays where it is.
    pub(crate) fn comments_above(&mut self, span: Span) {
        if !self.has_comments() {
            return;
        }
        let line_end = self.end_of_line(span.start());
        self.comments_before(line_end);
    }

    /// Writes every comment that starts before `offset`, each on a line of its own.
    pub(crate) fn comments_before(&mut self, offset: usize) {
        while let Some(span) = self.comments.get(self.written).copied() {
            if span.start() >= offset {
                return;
            }
            self.written += 1;
            self.open_line();
            self.word(span.text(self.source).trim_end());
            self.end_line();
        }
    }

    /// Whether any comment of the source is still waiting to be written.
    pub(crate) fn has_comments(&self) -> bool {
        self.written < self.comments.len()
    }

    /// Writes the indentation that opens a line.
    pub(crate) fn open_line(&mut self) {
        for _ in 0..self.depth {
            self.text.push_str(INDENT);
        }
    }

    /// `User`, or `demo.User`: a name, written against the module it is reached through.
    pub(crate) fn path(&mut self, written: &Path) {
        if let Some(module) = &written.module {
            self.word(&module.text);
            self.word(".");
        }
        self.word(&written.name.text);
    }

    pub(crate) fn word(&mut self, text: &str) {
        self.text.push_str(text);
    }

    pub(crate) fn end_line(&mut self) {
        self.text.push('\n');
    }

    pub(crate) fn blank_line(&mut self) {
        self.text.push('\n');
    }

    pub(crate) fn indent(&mut self) {
        self.depth += 1;
    }

    pub(crate) fn dedent(&mut self) {
        self.depth -= 1;
    }

    pub(crate) fn finish(self) -> String {
        self.text
    }

    /// The offset just past the end of the line `offset` sits on.
    fn end_of_line(&self, offset: usize) -> usize {
        self.source[offset..]
            .find('\n')
            .map_or(self.source.len(), |distance| offset + distance)
    }
}

/// The spans of the source's line comments, in source order.
fn comments_of(source: &str) -> Vec<Span> {
    lex(source)
        .into_iter()
        .filter(|token| token.kind == TokenKind::LineComment)
        .map(|token| token.span)
        .collect()
}
