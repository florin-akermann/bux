//! The edit a diagnostic carries where the compiler already knows what to write.

use lumen_lexer::Span;

/// An edit: the bytes a span covers become some other text.
///
/// `docs/specs/diagnostics.md` states what carries one and why. Applying it is a byte splice and
/// nothing more, so a tool that applies it needs to know nothing about the language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fix {
    span: Span,
    text: String,
}

impl Fix {
    /// The edit that writes `text` where `span` is.
    #[must_use]
    pub const fn new(span: Span, text: String) -> Self {
        Self { span, text }
    }

    /// Where the edit goes, as byte offsets into the file the diagnostic is about.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// What is written there, which is empty where the edit takes something out.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// `source` with this edit made, which is what a tool applying it ends up with.
    ///
    /// # Panics
    ///
    /// Panics when `source` is not the text the diagnostic was raised against, because a span
    /// into one file names nothing in another.
    #[must_use]
    pub fn applied_to(&self, source: &str) -> String {
        let mut edited = String::with_capacity(source.len());
        edited.push_str(&source[..self.span.start()]);
        edited.push_str(&self.text);
        edited.push_str(&source[self.span.end()..]);
        edited
    }
}
