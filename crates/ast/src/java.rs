//! The Java name an `extern` declaration states, as it was written.

use crate::Span;

/// One occurrence of a Java name, with the text it was spelled with.
///
/// It is a word here and nothing more. `java.nio.file.Files.readString` is its segments and the
/// dots between them, and whether the class it names is there at all is the JVM's answer rather
/// than the compiler's: `docs/specs/interop.md` states that nothing is loaded to refuse it
/// against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaName {
    pub text: String,
    pub span: Span,
}

impl JavaName {
    /// The segments the name is written as, which is what a class and a member are read out of.
    #[must_use]
    pub fn segments(&self) -> Vec<&str> {
        self.text.split('.').collect()
    }
}
