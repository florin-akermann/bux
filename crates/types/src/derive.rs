//! What a derive writes, and what a type has to hold before it writes anything.
//!
//! `docs/specs/derive.md` states the rule this checks: a derived instance reads field by field,
//! by the instance of each field's own type, so a field whose type has none leaves the derived
//! instance nothing to call. Nothing here infers anything — it reads what a module declares.

use lumen_ast::{Item, Name, RecordField, TypeDeclaration, TypeDefinition};
use lumen_ast::{TypeRef, Variant, VariantPayload};
use lumen_resolver::{ResolvedProgram, prelude};

use crate::environment::{Environment, signature_of};
use crate::error::{TypeError, TypeErrorKind};
use crate::types::Type;

/// One value a derived instance compares: where it sits, and the type it was written as.
struct Held<'a> {
    /// A field's name, or a variant's name and the name or position of what it carries.
    held_as: String,
    written: &'a TypeRef,
}

/// One instance a derive writes: the trait it names, the type it is for, and that declaration.
pub(crate) struct Writes<'a> {
    pub(crate) of: &'a Name,
    pub(crate) for_type: &'a Name,
    declared: &'a TypeDeclaration,
}

/// The type the one method of `of` has at the type called `named`, which is what a derive writes.
pub(crate) fn method_written(of: &str, named: &str) -> Type {
    let at = Type::Named {
        name: named.to_owned(),
        arguments: Vec::new(),
    };
    let method = prelude::method_of(of).expect("a derivable trait declares one method");
    signature_of(of, method, &at)
}

/// Refuses the first derive whose type holds a value of a type that has no instance of the trait.
///
/// This runs once every item has been declared, so a field whose type derives the trait further
/// down the file counts as having it and two types that hold each other derive it together.
///
/// # Errors
///
/// Returns the first value a deriving type holds whose own type has no instance of that trait.
pub(crate) fn hold_what_they_need(
    environment: &Environment,
    resolved: &ResolvedProgram,
) -> Result<(), TypeError> {
    for writes in written_in(resolved) {
        reads_what_it_holds(&writes)?;
        for held in holds(writes.declared) {
            holds_an_instance(environment, resolved, &writes, held)?;
        }
    }
    Ok(())
}

/// Refuses a derive of a type an `extern type` declares, whose contents are the JVM's.
///
/// `docs/specs/interop.md` states it: a derive reads what a type holds, and an extern type holds
/// what the JVM does, which an `instance` written over `extern` declarations is the answer to.
fn reads_what_it_holds(writes: &Writes<'_>) -> Result<(), TypeError> {
    if !matches!(&writes.declared.definition, TypeDefinition::Foreign(_)) {
        return Ok(());
    }
    let kind = TypeErrorKind::DerivesAForeignType(writes.for_type.text.clone());
    Err(TypeError::at(writes.for_type.span, kind))
}

/// Refuses one value a deriving type holds whose own type has no instance of the trait.
fn holds_an_instance(
    environment: &Environment,
    resolved: &ResolvedProgram,
    writes: &Writes<'_>,
    held: Held<'_>,
) -> Result<(), TypeError> {
    let of = environment.written(resolved, held.written)?;
    if has_instance(environment, &writes.of.text, &of) {
        return Ok(());
    }
    let kind = TypeErrorKind::HeldTypeHasNoInstance {
        of: writes.of.text.clone(),
        deriving: writes.for_type.text.clone(),
        held: of,
        held_as: held.held_as,
    };
    Err(TypeError::at(held.written.span, kind))
}

/// Every instance a module's derives write, with the declaration each one follows from.
pub(crate) fn written_in(resolved: &ResolvedProgram) -> impl Iterator<Item = Writes<'_>> {
    resolved
        .program()
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Derive(declaration) => Some(declaration),
            _ => None,
        })
        .flat_map(|declaration| {
            let declared = declared_as(resolved, &declaration.for_type.text);
            declaration.traits.iter().map(move |of| Writes {
                of,
                for_type: &declaration.for_type,
                declared,
            })
        })
}

/// Whether a value of `held` is ever handed to `of`, which is whether it has one to reach.
///
/// `docs/specs/traits.md` gives an instance to a type written by name, so a type written with
/// arguments has none however much its head does, and `()` has none at all.
fn has_instance(environment: &Environment, of: &str, held: &Type) -> bool {
    let Type::Named { name, arguments } = held else {
        return false;
    };
    arguments.is_empty() && environment.has_instance(of, name)
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

/// Every value a type holds, in the order a derived instance reads them.
fn holds(declared: &TypeDeclaration) -> Vec<Held<'_>> {
    match &declared.definition {
        // A Java class holds what the JVM holds, which no derived instance reads.
        TypeDefinition::Foreign(_) => Vec::new(),
        TypeDefinition::Record(fields) => fields.iter().map(field_of).collect(),
        TypeDefinition::Variants(variants) => variants.iter().flat_map(carried_by).collect(),
    }
}

/// One field of a record, which a derived instance reads by its own type's instance.
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
