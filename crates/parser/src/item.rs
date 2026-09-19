//! The top-level declarations of a source file.

use lumen_ast::{Function, Import, Item, Parameter, Program, RecordField, TypeDeclaration};
use lumen_ast::{TypeDefinition, TypeRef, Variant, VariantPayload};
use lumen_lexer::{Keyword, Punct, TokenKind};

use crate::cursor::Cursor;
use crate::error::{Expected, ParseError};
use crate::list::{comma_separated, newline_separated};
use crate::stmt::block;
use crate::type_ref::{type_parameters, type_ref};

/// Every item of the file, each ending at a newline or at the end of the file.
pub(crate) fn program(cursor: &mut Cursor) -> Result<Program, ParseError> {
    let mut items = Vec::new();
    while !cursor.at_end() {
        items.push(item(cursor)?);
        cursor.expect_end_of_line()?;
    }
    Ok(Program { items })
}

fn item(cursor: &mut Cursor) -> Result<Item, ParseError> {
    match cursor.peek_kind(0) {
        Some(TokenKind::Keyword(Keyword::Import)) => import(cursor).map(Item::Import),
        Some(TokenKind::Keyword(Keyword::Type)) => type_declaration(cursor).map(Item::Type),
        Some(TokenKind::Keyword(Keyword::Fn)) => function(cursor).map(Item::Function),
        _ => Err(cursor.error(Expected::Item)),
    }
}

fn import(cursor: &mut Cursor) -> Result<Import, ParseError> {
    let start = cursor.offset();
    cursor.expect_keyword(Keyword::Import)?;
    let module = cursor.expect_name(Expected::Name)?;
    Ok(Import {
        module,
        span: cursor.span_since(start),
    })
}

fn type_declaration(cursor: &mut Cursor) -> Result<TypeDeclaration, ParseError> {
    let start = cursor.offset();
    cursor.expect_keyword(Keyword::Type)?;
    let name = cursor.expect_name(Expected::Name)?;
    let parameters = type_parameters(cursor)?;
    cursor.expect_punct(Punct::Eq)?;
    let definition = type_definition(cursor)?;
    Ok(TypeDeclaration {
        name,
        parameters,
        definition,
        span: cursor.span_since(start),
    })
}

/// A record body, one variant on the same line, or a `|` variant per line.
fn type_definition(cursor: &mut Cursor) -> Result<TypeDefinition, ParseError> {
    if cursor.at_punct(Punct::LBrace) {
        return record_body(cursor).map(TypeDefinition::Record);
    }
    if !at_variant_bar(cursor) {
        return Ok(TypeDefinition::Variants(vec![variant(cursor)?]));
    }
    let mut variants = Vec::new();
    while at_variant_bar(cursor) {
        cursor.skip_newline();
        cursor.expect_punct(Punct::Pipe)?;
        variants.push(variant(cursor)?);
    }
    Ok(TypeDefinition::Variants(variants))
}

/// Whether another `|` variant follows, on this line or the next.
fn at_variant_bar(cursor: &Cursor) -> bool {
    cursor.at_punct(Punct::Pipe)
        || (cursor.at(TokenKind::Newline) && cursor.peek_kind(1) == Some(PIPE))
}

const PIPE: TokenKind = TokenKind::Punct(Punct::Pipe);

fn variant(cursor: &mut Cursor) -> Result<Variant, ParseError> {
    let start = cursor.offset();
    let name = cursor.expect_name(Expected::Name)?;
    let payload = variant_payload(cursor)?;
    Ok(Variant {
        name,
        payload,
        span: cursor.span_since(start),
    })
}

/// What a variant carries: nothing, positional types, or named fields.
fn variant_payload(cursor: &mut Cursor) -> Result<VariantPayload, ParseError> {
    if cursor.at_punct(Punct::LBrace) {
        return record_body(cursor).map(VariantPayload::Record);
    }
    if cursor.eat_punct(Punct::LParen).is_none() {
        return Ok(VariantPayload::None);
    }
    if cursor.at_punct(Punct::RParen) {
        return Err(cursor.error(Expected::Type));
    }
    comma_separated(cursor, Punct::RParen, type_ref).map(VariantPayload::Tuple)
}

/// `{ name: Type … }`, one field per line.
fn record_body(cursor: &mut Cursor) -> Result<Vec<RecordField>, ParseError> {
    cursor.expect_punct(Punct::LBrace)?;
    newline_separated(cursor, Punct::RBrace, record_field)
}

fn record_field(cursor: &mut Cursor) -> Result<RecordField, ParseError> {
    let start = cursor.offset();
    let name = cursor.expect_name(Expected::Name)?;
    cursor.expect_punct(Punct::Colon)?;
    let type_ref = type_ref(cursor)?;
    Ok(RecordField {
        name,
        type_ref,
        span: cursor.span_since(start),
    })
}

fn function(cursor: &mut Cursor) -> Result<Function, ParseError> {
    let start = cursor.offset();
    cursor.expect_keyword(Keyword::Fn)?;
    let name = cursor.expect_name(Expected::FunctionName)?;
    let type_parameters = type_parameters(cursor)?;
    cursor.expect_punct(Punct::LParen)?;
    let parameters = comma_separated(cursor, Punct::RParen, parameter)?;
    let result = result_type(cursor)?;
    let body = block(cursor)?;
    Ok(Function {
        name,
        type_parameters,
        parameters,
        result,
        body,
        span: cursor.span_since(start),
    })
}

/// A name, and the type annotation the author may have left to inference.
fn parameter(cursor: &mut Cursor) -> Result<Parameter, ParseError> {
    let start = cursor.offset();
    let name = cursor.expect_name(Expected::Name)?;
    let mut annotation = None;
    if cursor.eat_punct(Punct::Colon).is_some() {
        annotation = Some(type_ref(cursor)?);
    }
    Ok(Parameter {
        name,
        type_ref: annotation,
        span: cursor.span_since(start),
    })
}

/// The result type after `->`, which a function need not declare.
fn result_type(cursor: &mut Cursor) -> Result<Option<TypeRef>, ParseError> {
    if cursor.eat_punct(Punct::Arrow).is_none() {
        return Ok(None);
    }
    type_ref(cursor).map(Some)
}
