//! How a diagnostic reads, as `docs/specs/diagnostics.md` lays it out.

use lumen_lexer::Span;

use crate::Diagnostic;

/// `diagnostic` as the reader sees it, pointing into the `source` that came from `file`.
#[must_use]
pub fn render(diagnostic: &Diagnostic, source: &str, file: &str) -> String {
    let at = Place::of(diagnostic.span, source);
    let gutter = format!("  {} ", at.line);
    let mut lines = vec![
        format!(
            "error[{}]: {}",
            diagnostic.code.number(),
            diagnostic.message
        ),
        format!("  --> {file}:{}:{}", at.line, at.column),
        String::new(),
        format!("{gutter}| {}", at.text),
        format!(
            "{}| {}{}{}",
            " ".repeat(gutter.len()),
            at.padding(),
            "^".repeat(at.width),
            at.note()
        ),
    ];
    if let Some(help) = &diagnostic.help {
        lines.push(String::new());
        lines.push(format!("help: {help}"));
    }
    lines.join("\n") + "\n"
}

/// Where a span begins, as an editor counts it, and the line it begins on.
///
/// Only that one line is ever shown, however far the span runs, so that a diagnostic about a
/// deeply nested expression is as short as one about a misplaced comma.
struct Place<'a> {
    line: usize,
    column: usize,
    text: &'a str,
    /// How many characters of that line the span covers, which is never none.
    width: usize,
    /// The line the span ends on, when that is not the line it starts on.
    runs_on_to: Option<usize>,
}

impl<'a> Place<'a> {
    fn of(span: Span, source: &'a str) -> Self {
        let bytes = source.as_bytes();
        let start = span.start().min(source.len().saturating_sub(1));
        let opens = bytes[..start]
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |at| at + 1);
        let closes = bytes[start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(source.len(), |at| start + at);
        let line = line_at(source, start);
        let last = line_at(source, span.end().min(source.len()).saturating_sub(1));
        Self {
            line,
            column: characters_between(source, opens, start) + 1,
            text: &source[opens..closes],
            width: characters_between(source, start, span.end().min(closes)).max(1),
            runs_on_to: (last > line).then_some(last),
        }
    }

    /// What the caret row writes before the carets, so that the two line up.
    ///
    /// A tab is written back as a tab rather than as one blank, because the width of a tab
    /// is the terminal's to decide and the caret row has to agree with the line above it.
    fn padding(&self) -> String {
        self.text
            .chars()
            .take(self.column - 1)
            .map(|written| if written == '\t' { '\t' } else { ' ' })
            .collect()
    }

    /// What the caret row says about a span that does not end on the line it starts on.
    fn note(&self) -> String {
        self.runs_on_to
            .map_or_else(String::new, |line| format!(" this runs on to line {line}"))
    }
}

/// How many characters of `source` lie between the two byte offsets.
///
/// This asks the source rather than slicing it, so an offset in the middle of a character is
/// still a question it can answer, and a span nobody lexed cannot panic the renderer.
fn characters_between(source: &str, from: usize, to: usize) -> usize {
    source
        .char_indices()
        .filter(|(at, _)| *at >= from && *at < to)
        .count()
}

/// The one-based line the byte at `offset` sits on.
///
/// This counts bytes rather than slicing, so an offset in the middle of a character is still a
/// question it can answer.
fn line_at(source: &str, offset: usize) -> usize {
    source
        .bytes()
        .take(offset)
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}
