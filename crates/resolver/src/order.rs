//! Where a declaration belongs, which is above what it uses.
//!
//! `docs/design.md` section 13 has a file read top down: the reader meets the intent before the
//! detail, so a declaration is written above what it uses and a helper below the thing it helps.
//! The rule is checked here because this is the phase that knows which name means which
//! declaration; nothing is moved, and the refusal says where the declaration belongs.
//!
//! Two declarations that use each other are written either way, because no order undoes a cycle.
//! A use is therefore only out of order when what it uses cannot reach back to it.

use std::collections::{BTreeSet, HashMap};

use lumen_ast::{Item, Name, Program, Span};

use crate::definition::{Definition, Namespace, Origin};
use crate::error::{ResolveError, ResolveErrorKind};

/// The first declaration of `program` written above something that uses it, when one is.
pub(crate) fn out_of_order(
    program: &Program,
    definitions: &HashMap<(Namespace, Span), Definition>,
) -> Option<ResolveError> {
    let declared = declarations(program);
    let uses = uses(&declared, definitions);
    for &(meant, writes) in &uses {
        if writes < meant || reaches(&uses, meant, writes) {
            continue;
        }
        return Some(ResolveError::at(
            declared[meant].name,
            ResolveErrorKind::written_above(&declared[meant].written, &declared[writes].written),
        ));
    }
    None
}

/// One declaration of a module: the name it declares, how it is spoken of, and how far it
/// reaches in the source.
struct Declared<'a> {
    name: &'a Name,
    /// What a refusal calls this declaration, which is its name for all but an instance.
    written: String,
    extent: Span,
}

/// Every declaration of the module in source order, which is every item but an import.
///
/// An import declares nothing this rule places: `docs/design.md` section 13 puts every import
/// first and sorted, which settles where it goes without asking what uses it.
fn declarations(program: &Program) -> Vec<Declared<'_>> {
    program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Import(_) => None,
            Item::Type(declaration) => Some(named(&declaration.name, declaration.span)),
            Item::Trait(declaration) => Some(named(&declaration.name, declaration.span)),
            // An instance declares no name, so it is spoken of by the trait and the type it
            // names: it is written below that trait, and below whatever its bodies reach.
            Item::Instance(declaration) => Some(Declared {
                name: &declaration.trait_name,
                written: format!(
                    "{}<{}>",
                    declaration.trait_name.text, declaration.for_type.text
                ),
                extent: declaration.span,
            }),
            Item::Function(function) => Some(named(&function.name, function.span)),
        })
        .collect()
}

/// Which declaration uses which, as `(meant, writes)` pairs: the declaration a name means, and
/// the declaration the name is written in.
///
/// They come out of a map, whose order is nobody's, so they are gathered into a sorted set: the
/// refusal a run reports has to be the same one every run reports.
fn uses(
    declared: &[Declared<'_>],
    definitions: &HashMap<(Namespace, Span), Definition>,
) -> BTreeSet<(usize, usize)> {
    let mut found = BTreeSet::new();
    for ((_, written), definition) in definitions {
        let Origin::Declared(at) = definition.origin else {
            continue;
        };
        let (Some(writes), Some(meant)) = (holding(declared, *written), holding(declared, at))
        else {
            continue;
        };
        if writes != meant {
            found.insert((meant, writes));
        }
    }
    found
}

/// A declaration a refusal speaks of by the one name it declares.
fn named(name: &Name, extent: Span) -> Declared<'_> {
    Declared {
        name,
        written: name.text.clone(),
        extent,
    }
}

/// Which declaration the source at `written` is inside, when it is inside one at all.
fn holding(declared: &[Declared<'_>], written: Span) -> Option<usize> {
    declared
        .iter()
        .position(|declaration| declaration.extent.range().contains(&written.start()))
}

/// Whether `from` uses `to`, however many declarations it takes to get there.
fn reaches(uses: &BTreeSet<(usize, usize)>, from: usize, to: usize) -> bool {
    let mut seen = BTreeSet::from([from]);
    let mut walking = vec![from];
    while let Some(at) = walking.pop() {
        for &(meant, writes) in uses {
            if writes == at && seen.insert(meant) {
                if meant == to {
                    return true;
                }
                walking.push(meant);
            }
        }
    }
    false
}
