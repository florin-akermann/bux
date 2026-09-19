//! The two ways the grammar writes a list: separated by commas, or by newlines.

use lumen_lexer::Punct;

use crate::cursor::Cursor;
use crate::error::ParseError;

/// Elements up to and including `close`, separated by commas and free to span lines.
///
/// The opening bracket has already been consumed. A trailing comma is not canonical form and is
/// not accepted, so `close` must follow the last element.
pub(crate) fn comma_separated<T>(
    cursor: &mut Cursor,
    close: Punct,
    mut element: impl FnMut(&mut Cursor) -> Result<T, ParseError>,
) -> Result<Vec<T>, ParseError> {
    let mut elements = Vec::new();
    cursor.skip_newline();
    if cursor.eat_punct(close).is_some() {
        return Ok(elements);
    }
    loop {
        elements.push(element(cursor)?);
        cursor.skip_newline();
        if cursor.eat_punct(close).is_some() {
            return Ok(elements);
        }
        cursor.expect_punct(Punct::Comma)?;
        cursor.skip_newline();
    }
}

/// Elements up to and including `close`, one per line.
///
/// The opening `{` has already been consumed. Each element but the last ends at a newline, and
/// the last may end at one too.
pub(crate) fn newline_separated<T>(
    cursor: &mut Cursor,
    close: Punct,
    mut element: impl FnMut(&mut Cursor) -> Result<T, ParseError>,
) -> Result<Vec<T>, ParseError> {
    let mut elements = Vec::new();
    loop {
        if cursor.eat_punct(close).is_some() {
            return Ok(elements);
        }
        if cursor.at_end() {
            return Err(cursor.error(crate::error::Expected::Punct(close)));
        }
        elements.push(element(cursor)?);
        if !cursor.at_punct(close) {
            cursor.expect_end_of_line()?;
        }
    }
}
