//! The top-level declarations of a source file.

use lumen_ast::{DeriveDeclaration, Function, Import, InstanceDeclaration, Item, Name};
use lumen_ast::{Parameter, Program};
use lumen_ast::{RecordField, Signature, TraitDeclaration, TypeDeclaration, TypeDefinition};
use lumen_ast::{Span, TypeRef, Variant, VariantPayload};
use lumen_lexer::{Keyword, Punct, TokenKind};

use crate::cursor::Cursor;
use crate::error::{Expected, ParseError};
use crate::list::{Emptiness, comma_separated, newline_separated};
use crate::stmt::block;
use crate::type_ref::{constrained_parameters, type_parameters, type_ref};

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
        Some(TokenKind::Keyword(Keyword::Trait)) => trait_declaration(cursor).map(Item::Trait),
        Some(TokenKind::Keyword(Keyword::Instance)) => instance(cursor).map(Item::Instance),
        Some(TokenKind::Keyword(Keyword::Derive)) => derive(cursor).map(Item::Derive),
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
    comma_separated(cursor, Punct::RParen, Emptiness::Forbidden, type_ref)
        .map(VariantPayload::Tuple)
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

/// `trait Eq<T> { … }`: a name, the one type it is written over, and what a type can then do.
fn trait_declaration(cursor: &mut Cursor) -> Result<TraitDeclaration, ParseError> {
    let read = over_one_type(cursor, Keyword::Trait, Named::TRAIT, signature)?;
    Ok(TraitDeclaration {
        name: read.name,
        parameter: read.argument,
        methods: read.body,
        span: read.span,
    })
}

/// A function's first line with no body, which is what a trait declares a method as.
fn signature(cursor: &mut Cursor) -> Result<Signature, ParseError> {
    let start = cursor.offset();
    cursor.expect_keyword(Keyword::Fn)?;
    let name = cursor.expect_name(Expected::FunctionName)?;
    let (parameters, result) = takes_and_gives(cursor)?;
    Ok(Signature {
        name,
        parameters,
        result,
        span: cursor.span_since(start),
    })
}

/// `instance Eq<Point> { … }`: a trait, the type it is for, and a body for each of its methods.
fn instance(cursor: &mut Cursor) -> Result<InstanceDeclaration, ParseError> {
    let read = over_one_type(cursor, Keyword::Instance, Named::INSTANCE, function)?;
    Ok(InstanceDeclaration {
        trait_name: read.name,
        for_type: read.argument,
        methods: read.body,
        span: read.span,
    })
}

/// `derive Eq, Ord for User`: the traits the compiler writes, and the type it writes them for.
fn derive(cursor: &mut Cursor) -> Result<DeriveDeclaration, ParseError> {
    let start = cursor.offset();
    cursor.expect_keyword(Keyword::Derive)?;
    let mut traits = vec![cursor.expect_name(Expected::Name)?];
    while cursor.eat_punct(Punct::Comma).is_some() {
        traits.push(cursor.expect_name(Expected::Name)?);
    }
    cursor.expect_keyword(Keyword::For)?;
    let for_type = cursor.expect_name(Expected::Name)?;
    Ok(DeriveDeclaration {
        traits,
        for_type,
        span: cursor.span_since(start),
    })
}

fn function(cursor: &mut Cursor) -> Result<Function, ParseError> {
    let start = cursor.offset();
    cursor.expect_keyword(Keyword::Fn)?;
    let name = cursor.expect_name(Expected::FunctionName)?;
    let type_parameters = constrained_parameters(cursor)?;
    let (parameters, result) = takes_and_gives(cursor)?;
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

/// A trait and an instance are one shape: a keyword, `Name<Name>`, and a body of declarations.
struct OverOneType<T> {
    name: Name,
    argument: Name,
    body: Vec<T>,
    span: Span,
}

/// What the reader was meant to write on either side of the angles, for each of the two.
#[derive(Clone, Copy)]
struct Named {
    outside: Expected,
    inside: Expected,
}

impl Named {
    /// A trait names itself, and names the one type parameter its methods are written over.
    const TRAIT: Self = Self {
        outside: Expected::Name,
        inside: Expected::Name,
    };
    /// An instance names the trait it is for, and the type that is giving it one.
    const INSTANCE: Self = Self {
        outside: Expected::Trait,
        inside: Expected::Type,
    };
}

/// `trait Eq<T> { … }` or `instance Eq<Point> { … }`, read as the shape the two share.
fn over_one_type<T>(
    cursor: &mut Cursor,
    opener: Keyword,
    named: Named,
    declaration: impl FnMut(&mut Cursor) -> Result<T, ParseError>,
) -> Result<OverOneType<T>, ParseError> {
    let start = cursor.offset();
    cursor.expect_keyword(opener)?;
    let name = cursor.expect_name(named.outside)?;
    cursor.expect_punct(Punct::Lt)?;
    let argument = cursor.expect_name(named.inside)?;
    cursor.expect_punct(Punct::Gt)?;
    let body = body_of(cursor, declaration)?;
    Ok(OverOneType {
        name,
        argument,
        body,
        span: cursor.span_since(start),
    })
}

/// `(one: Int, other: Int) -> Bool`: what a function takes, and what it gives back.
///
/// A signature and a function are the same from the `(` on, which is what this is.
fn takes_and_gives(cursor: &mut Cursor) -> Result<(Vec<Parameter>, Option<TypeRef>), ParseError> {
    cursor.expect_punct(Punct::LParen)?;
    let parameters = comma_separated(cursor, Punct::RParen, Emptiness::Allowed, parameter)?;
    let result = result_type(cursor)?;
    Ok((parameters, result))
}

/// The `{ … }` of a trait or an instance: one declaration per line, and never none at all.
///
/// A body holding nothing reads as the declaration that was wanted and is not there, which is
/// what the `}` is reported against.
fn body_of<T>(
    cursor: &mut Cursor,
    mut declaration: impl FnMut(&mut Cursor) -> Result<T, ParseError>,
) -> Result<Vec<T>, ParseError> {
    cursor.expect_punct(Punct::LBrace)?;
    let mut declared = Vec::new();
    loop {
        cursor.skip_newline();
        if !declared.is_empty() && cursor.eat_punct(Punct::RBrace).is_some() {
            return Ok(declared);
        }
        declared.push(declaration(cursor)?);
    }
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
