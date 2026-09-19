//! Byte spans into the source text.

use std::ops::Range;

/// A byte range into the source, held as an offset and a length.
///
/// Building a span from an offset and a length means an end before its start cannot be written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    start: usize,
    len: usize,
}

impl Span {
    /// The span of `len` bytes beginning at byte offset `start`.
    #[must_use]
    pub const fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    /// The byte offset of the first byte in the span.
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// How many bytes the span covers, which is what a tool reading it as data is given.
    #[must_use]
    pub const fn bytes(self) -> usize {
        self.len
    }

    /// The source text the span names.
    #[must_use]
    pub fn text(self, source: &str) -> &str {
        &source[self.range()]
    }

    /// The span as a half-open byte range, for slicing the source.
    #[must_use]
    pub const fn range(self) -> Range<usize> {
        self.start..self.end()
    }

    /// The byte offset one past the last byte in the span.
    #[must_use]
    pub const fn end(self) -> usize {
        self.start + self.len
    }
}
