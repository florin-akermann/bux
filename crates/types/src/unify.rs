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
fn settle(table: &mut Table, var: TypeVar, to: &Type) -> Result<(), Clash> {
    let mut held = Vec::new();
    table.unsettled(to, &mut held);
    if held.contains(&var) {
        return Err(Clash::Infinite);
    }
    table.settle(var, to.clone());
    Ok(())
}

fn each(table: &mut Table, theirs: &[Type], ours: &[Type]) -> Result<(), Clash> {
    for (one, other) in theirs.iter().zip(ours) {
        unify(table, one, other)?;
    }
    Ok(())
}
