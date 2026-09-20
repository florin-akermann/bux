//! Whether a declared type holds a value of itself, which no whole value could be.
//!
//! `docs/specs/types.md` states the rule. A record's field of record type is laid out inside it,
//! so a ring of those has no first value to build and no end to build towards. Nothing here
//! infers anything: it reads what a module declares and refuses the first ring it finds.

use std::collections::HashMap;

use lumen_ast::{Item, Name, RecordField, Span, TypeDefinition, TypeRef, TypeRefKind};
use lumen_resolver::ResolvedProgram;

use crate::error::{TypeError, TypeErrorKind};

/// Which records each record holds by value, by the name each is declared under.
type Holds = HashMap<String, Vec<String>>;

/// Refuses the first declared type that holds a value of itself.
///
/// # Errors
///
/// Returns the first ring of holdings a walk of the declarations comes back round to.
pub(crate) fn nothing_holds_itself(resolved: &ResolvedProgram) -> Result<(), TypeError> {
    let declared = records_of(resolved);
    let holds = held_by(&declared);
    let mut walk = Walk::default();
    for (name, _) in &declared {
        if walk.is_walked(&name.text) {
            continue;
        }
        if let Some(ring) = walk.ring_from(&name.text, &holds) {
            let at = declared_at(&declared, &ring[0]);
            return Err(TypeError::at(at, TypeErrorKind::HoldsItself { ring }));
        }
    }
    Ok(())
}

/// Every record a module declares, in the order it declares them, with the fields of each.
///
/// A record is the only declaration that holds another by value. A field written as the base of
/// an algebraic data type holds whichever variant it was handed, which is a reference, and so a
/// ring through one is finite and is not one.
fn records_of(resolved: &ResolvedProgram) -> Vec<(&Name, &[RecordField])> {
    let mut declared = Vec::new();
    for item in &resolved.program().items {
        let Item::Type(declaration) = item else {
            continue;
        };
        let TypeDefinition::Record(fields) = &declaration.definition else {
            continue;
        };
        declared.push((&declaration.name, fields.as_slice()));
    }
    declared
}

/// The records each one holds by value, which is what a ring runs along.
fn held_by(declared: &[(&Name, &[RecordField])]) -> Holds {
    let records: Vec<&str> = declared
        .iter()
        .map(|(name, _)| name.text.as_str())
        .collect();
    declared
        .iter()
        .map(|(name, fields)| {
            let held = fields
                .iter()
                .filter_map(|field| head_of(&field.type_ref))
                .filter(|held| records.contains(&held.as_str()))
                .collect();
            (name.text.clone(), held)
        })
        .collect()
}

/// Where `named` is declared, which is where the ring it opens is reported.
fn declared_at(declared: &[(&Name, &[RecordField])], named: &str) -> Span {
    declared
        .iter()
        .find(|(name, _)| name.text == named)
        .map(|(name, _)| name.span)
        .expect("a ring runs through records this module declares")
}

/// The name a written type applies its arguments to, where it is a named type.
///
/// An argument is not looked into: a type argument is carried as a reference, so a ring that
/// runs through one holds nothing of itself and is finite.
fn head_of(written: &TypeRef) -> Option<String> {
    match &written.kind {
        TypeRefKind::Named { name, .. } => Some(name.text.clone()),
        TypeRefKind::Unit => None,
    }
}

/// How far a walk has got with one record.
enum Reached {
    /// The walk is inside it: anything reaching it again has come back round.
    OnTheWayIn,
    /// The walk left it having found no ring, so nothing need go back in.
    Left,
}

/// One walk of the declarations, which goes into each record once however many hold it.
#[derive(Default)]
struct Walk {
    reached: HashMap<String, Reached>,
    path: Vec<String>,
}

impl Walk {
    /// Whether this walk has already been into `named` and come out again.
    fn is_walked(&self, named: &str) -> bool {
        self.reached.contains_key(named)
    }

    /// The ring the walk from `at` comes back round to, where it comes back to one.
    fn ring_from(&mut self, at: &str, holds: &Holds) -> Option<Vec<String>> {
        self.reached.insert(at.to_owned(), Reached::OnTheWayIn);
        self.path.push(at.to_owned());
        for held in holds.get(at).into_iter().flatten() {
            match self.reached.get(held) {
                Some(Reached::OnTheWayIn) => return Some(self.ring_back_to(held)),
                Some(Reached::Left) => {}
                None => {
                    if let Some(ring) = self.ring_from(held, holds) {
                        return Some(ring);
                    }
                }
            }
        }
        self.reached.insert(at.to_owned(), Reached::Left);
        self.path.pop();
        None
    }

    /// The ring the path holds: from where `held` was gone into, round to where it came up again.
    fn ring_back_to(&self, held: &str) -> Vec<String> {
        let from = self
            .path
            .iter()
            .position(|walked| walked == held)
            .expect("the walk is inside it, so the way in holds it");
        self.path[from..].to_vec()
    }
}
