//! Types as they are written in source.

use lumen_ast::{TypeRef, TypeRefKind};
use lumen_lexer::Punct;

use crate::cursor::Cursor;
use crate::error::{Expected, ParseError};
use crate::list::{Emptiness, comma_separated};

/// `Int`, `List<User>`, or `()`.
pub(crate) fn type_ref(cursor: &mut Cursor) -> Result<TypeRef, ParseError> {
    let start = cursor.offset();
    if cursor.eat_punct(Punct::LParen).is_some() {
        cursor.expect_punct(Punct::RParen)?;
        return Ok(TypeRef {
            kind: TypeRefKind::Unit,
            span: cursor.span_since(start),
        });
    }
    let name = cursor.expect_name(Expected::Type)?;
    let arguments = type_arguments(cursor)?;
    Ok(TypeRef {
        kind: TypeRefKind::Named { name, arguments },
        span: cursor.span_since(start),
    })
}

/// The `<…>` after a type name, which most types do not have.
fn type_arguments(cursor: &mut Cursor) -> Result<Vec<TypeRef>, ParseError> {
    if cursor.eat_punct(Punct::Lt).is_none() {
        return Ok(Vec::new());
    }
    cursor.nested(|cursor| comma_separated(cursor, Punct::Gt, Emptiness::Forbidden, type_ref))
}

/// The `<T, E>` after a type or function name, naming the types it is generic over.
pub(crate) fn type_parameters(cursor: &mut Cursor) -> Result<Vec<lumen_ast::Name>, ParseError> {
    if cursor.eat_punct(Punct::Lt).is_none() {
        return Ok(Vec::new());
    }
    comma_separated(cursor, Punct::Gt, Emptiness::Forbidden, |cursor| {
        cursor.expect_name(Expected::Name)
    })
}
