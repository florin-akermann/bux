//! Making two types one type, or saying that they are not.

use crate::table::Table;
use crate::types::{Type, TypeVar};

/// Makes `expected` and `found` the same type, settling whatever variables that takes.
///
/// # Errors
///
/// Returns the way the two failed to meet: they are different types, or one would contain itself.
pub(crate) fn unify(table: &mut Table, expected: &Type, found: &Type) -> Result<(), Clash> {
    let expected = table.shallow(expected);
    let found = table.shallow(found);
    match (&expected, &found) {
        (Type::Var(one), Type::Var(other)) if one == other => Ok(()),
        (Type::Var(var), other) | (other, Type::Var(var)) => settle(table, *var, other),
        (Type::Unit, Type::Unit) => Ok(()),
        (Type::Parameter(one), Type::Parameter(other)) if one == other => Ok(()),
        (Type::Module(one), Type::Module(other)) if one == other => Ok(()),
        (
            Type::Named {
                name: one,
                arguments: theirs,
            },
            Type::Named {
                name: other,
                arguments: ours,
            },
        ) if one == other && theirs.len() == ours.len() => each(table, theirs, ours),
        (
            Type::Function {
                parameters: theirs,
                result: one,
            },
            Type::Function {
                parameters: ours,
                result: other,
            },
        ) if theirs.len() == ours.len() => {
            each(table, theirs, ours)?;
            unify(table, one, other)
        }
        _ => Err(Clash::Mismatch),
    }
}

/// The two ways unification fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Clash {
    /// The two are different types.
    Mismatch,
    /// One of the two would have to contain itself.
    Infinite,
}

/// Settles `var` on `to`, unless `to` is a type that holds `var` and so cannot be finite.
///
/// A variable standing for a whole number settles on less than an ordinary one: only on a type a
/// whole number may be written at. A clash here is therefore the ordinary clash of two types, and
/// is reported as one, because such a variable reads as the `Int` it would have defaulted to.
fn settle(table: &mut Table, var: TypeVar, to: &Type) -> Result<(), Clash> {
    let mut held = Vec::new();
    table.unsettled(to, &mut held);
    if held.contains(&var) {
        return Err(Clash::Infinite);
    }
    if table.is_a_whole_number(var) && !stands_for_a_whole_number(table, to) {
        return Err(Clash::Mismatch);
    }
    table.settle(var, to.clone());
    Ok(())
}

/// Whether a whole number may be what `to` stands for, which settling one on it needs.
///
/// A variable may: two whole numbers meeting are one whole number, and whatever settles the one
/// that is left settles both. Everything else is a type, and takes a whole number or does not.
fn stands_for_a_whole_number(table: &mut Table, to: &Type) -> bool {
    if let Type::Var(other) = to {
        table.note_a_whole_number(*other);
        return true;
    }
    table.takes_a_whole_number(to)
}

fn each(table: &mut Table, theirs: &[Type], ours: &[Type]) -> Result<(), Clash> {
    for (one, other) in theirs.iter().zip(ours) {
        unify(table, one, other)?;
    }
    Ok(())
}
