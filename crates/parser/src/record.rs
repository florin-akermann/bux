//! Record literals: building a record, and updating one.
//!
//! `User { id: id }` and `user { name: "Bob" }` are the same node. Which one a source line
//! means depends on what its base name refers to, which only name resolution knows.

use lumen_ast::{ExprKind, FieldValue, Name};
use lumen_lexer::Punct;

use crate::cursor::Cursor;
use crate::error::{Expected, ParseError};
use crate::expr::{RecordLiterals, expression};
use crate::list::comma_separated;

/// The `{ … }` after a base name, whose opening brace has not been consumed.
pub(crate) fn record_literal(cursor: &mut Cursor, base: Name) -> Result<ExprKind, ParseError> {
    cursor.expect_punct(Punct::LBrace)?;
    let fields = comma_separated(cursor, Punct::RBrace, field_value)?;
    Ok(ExprKind::Record { base, fields })
}

fn field_value(cursor: &mut Cursor) -> Result<FieldValue, ParseError> {
    let start = cursor.offset();
    let name = cursor.expect_name(Expected::Name)?;
    cursor.expect_punct(Punct::Colon)?;
    let value = expression(cursor, RecordLiterals::Allowed)?;
    Ok(FieldValue {
        name,
        value,
        span: cursor.span_since(start),
    })
}
