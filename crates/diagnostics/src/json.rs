//! A diagnostic as data, which `docs/specs/diagnostics.md` states field for field.

use std::fmt::Write as _;

use lumen_lexer::Span;

use crate::{Diagnostic, Fix};

/// `diagnostic` as one line of JSON, about the file named `file`.
///
/// The line ends with a newline, so one refusal per line is what a reader of the stream gets.
#[must_use]
pub fn json(diagnostic: &Diagnostic, file: &str) -> String {
    let mut fields = vec![
        text("file", file),
        text("code", diagnostic.code.number()),
        text("message", &diagnostic.message),
        format!("\"span\":{}", where_it_is(diagnostic.span)),
    ];
    if let Some(help) = &diagnostic.help {
        fields.push(text("help", help));
    }
    if let Some(fix) = &diagnostic.fix {
        fields.push(format!("\"fix\":{}", edit(fix)));
    }
    format!("{{{}}}\n", fields.join(","))
}

/// `"name":"value"`, with the value written the way JSON writes text.
fn text(name: &str, value: &str) -> String {
    format!("\"{name}\":{}", quoted(value))
}

/// A span as the two numbers a tool splices with.
fn where_it_is(span: Span) -> String {
    format!("{{\"start\":{},\"len\":{}}}", span.start(), span.bytes())
}

/// An edit as where it goes and what goes there.
fn edit(fix: &Fix) -> String {
    let span = fix.span();
    format!(
        "{{\"start\":{},\"len\":{},\"text\":{}}}",
        span.start(),
        span.bytes(),
        quoted(fix.text())
    )
}

/// `written` as a JSON string, with everything JSON does not write plainly escaped.
///
/// A message is one line by the time it reaches here, but a message quotes the source, and the
/// source holds whatever the author wrote; a quote or a newline in it must not end the line.
fn quoted(written: &str) -> String {
    let mut quoted = String::with_capacity(written.len() + 2);
    quoted.push('"');
    for character in written.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            control if control < ' ' => {
                write!(quoted, "\\u{:04x}", control as u32).expect("a String takes every write");
            }
            plain => quoted.push(plain),
        }
    }
    quoted.push('"');
    quoted
}
