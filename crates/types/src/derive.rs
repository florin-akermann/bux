//! What a derive writes, and what a type has to hold before it writes anything.
//!
//! `docs/specs/derive.md` states the rule this checks: a derived `Eq` compares field by field,
//! by the `Eq` of each field's own type, so a field whose type has none leaves the derived
//! instance nothing to call. Nothing here infers anything — it reads what a module declares.

use lumen_ast::{DeriveDeclaration, Item, RecordField, TypeDeclaration, TypeDefinition};
use lumen_ast::{TypeRef, Variant, VariantPayload};
use lumen_resolver::{ResolvedProgram, prelude};

use crate::environment::Environment;
use crate::error::{TypeError, TypeErrorKind};
use crate::types::Type;

/// One value a derived instance compares: where it sits, and the type it was written as.
struct Held<'a> {
    /// A field's name, or a variant's name and the name or position of what it carries.
    held_as: String,
    written: &'a TypeRef,
}

/// The type `Eq`'s method has at the type called `named`, which is what a derive writes.
pub(crate) fn is_equal_at(named: &str) -> Type {
    let at = Type::Named {
        name: named.to_owned(),
        arguments: Vec::new(),
    };
    Type::function(vec![at.clone(), at], Type::boolean())
}

/// Refuses the first derive whose type holds a value of a type that has no `Eq`.
///
/// This runs once every item has been declared, so a field whose type derives `Eq` further down
/// the file counts as having one and two types that hold each other derive `Eq` together.
///
/// # Errors
///
/// Returns the first value a deriving type holds whose own type has no instance of `Eq`.
pub(crate) fn compare_what_they_hold(
    environment: &Environment,
    resolved: &ResolvedProgram,
) -> Result<(), TypeError> {
    for (declaration, declared) in written_in(resolved) {
        for held in holds(declared) {
            let of = environment.written(resolved, held.written)?;
            if has_equality(environment, &of) {
                continue;
            }
            let kind = TypeErrorKind::HeldTypeHasNoInstance {
                deriving: declaration.for_type.text.clone(),
                held: of,
                held_as: held.held_as,
            };
            return Err(TypeError::at(held.written.span, kind));
        }
    }
    Ok(())
}

/// Every derive a module writes, with the declaration of the type each one names.
pub(crate) fn written_in(
    resolved: &ResolvedProgram,
) -> impl Iterator<Item = (&DeriveDeclaration, &TypeDeclaration)> {
    resolved
        .program()
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Derive(declaration) => Some(declaration),
            _ => None,
        })
        .map(|declaration| {
            (
                declaration,
                declared_as(resolved, &declaration.for_type.text),
            )
        })
}

/// Whether two values of `of` are ever compared, which is whether the type has an `Eq` to reach.
///
/// `docs/specs/traits.md` gives an instance to a type written by name, so a type written with
/// arguments has none however much its head does, and `()` has none at all.
fn has_equality(environment: &Environment, of: &Type) -> bool {
    let Type::Named { name, arguments } = of else {
        return false;
    };
    arguments.is_empty() && environment.has_instance(prelude::EQ, name)
}

/// The declaration of the type called `named`, which is the one the derive above it names.
fn declared_as<'a>(resolved: &'a ResolvedProgram, named: &str) -> &'a TypeDeclaration {
    resolved
        .program()
        .items
        .iter()
        .find_map(|item| match item {
            Item::Type(declaration) if declaration.name.text == named => Some(declaration),
            _ => None,
        })
        .expect("name resolution gave the derive's type a declaration in this module")
}

/// Every value a type holds, in the order a derived instance compares them.
fn holds(declared: &TypeDeclaration) -> Vec<Held<'_>> {
    match &declared.definition {
        TypeDefinition::Record(fields) => fields.iter().map(field_of).collect(),
        TypeDefinition::Variants(variants) => variants.iter().flat_map(carried_by).collect(),
    }
}

/// One field of a record, which a derived instance compares by its own type's `Eq`.
fn field_of(field: &RecordField) -> Held<'_> {
    Held {
        held_as: field.name.text.clone(),
        written: &field.type_ref,
    }
}

/// What one variant carries, each named for the variant and for its own place in it.
fn carried_by(variant: &Variant) -> Vec<Held<'_>> {
    let named = &variant.name.text;
    match &variant.payload {
        VariantPayload::None => Vec::new(),
        VariantPayload::Tuple(written) => written
            .iter()
            .enumerate()
            .map(|(position, type_ref)| Held {
                held_as: format!("{named}.{position}"),
                written: type_ref,
            })
            .collect(),
        VariantPayload::Record(fields) => fields
            .iter()
            .map(|field| Held {
                held_as: format!("{named}.{}", field.name.text),
                written: &field.type_ref,
            })
            .collect(),
    }
}
