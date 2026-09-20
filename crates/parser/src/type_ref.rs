//! Types as they are written in source.

use lumen_ast::{Constraint, TypeParameter, TypeRef, TypeRefKind};
use lumen_lexer::Punct;

use crate::cursor::Cursor;
use crate::error::{Expected, ParseError};
use crate::list::{Emptiness, comma_separated};

/// `Int`, `List<User>`, `demo.Held<Int>`, or `()`.
pub(crate) fn type_ref(cursor: &mut Cursor) -> Result<TypeRef, ParseError> {
    let start = cursor.offset();
    if cursor.eat_punct(Punct::LParen).is_some() {
        cursor.expect_punct(Punct::RParen)?;
        return Ok(TypeRef {
            kind: TypeRefKind::Unit,
            span: cursor.span_since(start),
        });
    }
    let path = cursor.expect_path(Expected::Type)?;
    let arguments = type_arguments(cursor)?;
    Ok(TypeRef {
        kind: TypeRefKind::Named { path, arguments },
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

/// The `<T, E>` or `<T: Eq<T>>` after a function name, naming the types it is written over.
pub(crate) fn constrained_parameters(
    cursor: &mut Cursor,
) -> Result<Vec<TypeParameter>, ParseError> {
    if cursor.eat_punct(Punct::Lt).is_none() {
        return Ok(Vec::new());
    }
    comma_separated(
        cursor,
        Punct::Gt,
        Emptiness::Forbidden,
        constrained_parameter,
    )
}

/// One type parameter, with the trait it is constrained by when the author wrote one.
fn constrained_parameter(cursor: &mut Cursor) -> Result<TypeParameter, ParseError> {
    let name = cursor.expect_name(Expected::Name)?;
    if cursor.eat_punct(Punct::Colon).is_none() {
        return Ok(TypeParameter {
            name,
            constraint: None,
        });
    }
    let constraint = constraint(cursor)?;
    Ok(TypeParameter {
        name,
        constraint: Some(constraint),
    })
}

/// `Eq<T>`: a trait, written over the one type it constrains.
fn constraint(cursor: &mut Cursor) -> Result<Constraint, ParseError> {
    let start = cursor.offset();
    let name = cursor.expect_name(Expected::Trait)?;
    cursor.expect_punct(Punct::Lt)?;
    let argument = cursor.nested(type_ref)?;
    cursor.expect_punct(Punct::Gt)?;
    Ok(Constraint {
        name,
        argument,
        span: cursor.span_since(start),
    })
}
