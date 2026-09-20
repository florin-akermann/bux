//! What an instance of `IntegerLiteral` says its type holds, which is read rather than run.
//!
//! `docs/specs/literals.md` states the restriction: the body of `lowest` and of `highest` is one
//! whole number and nothing else, optionally with a `-` in front, which the parser reads as the
//! one negative number it spells. The compiler reads those two bodies where it reads any other
//! declaration, so a literal that does not fit is refused while the program is compiled and never
//! at run time.

use lumen_ast::{ExprKind, Function, InstanceDeclaration, StatementKind};
use lumen_resolver::prelude;

use crate::error::{TypeError, TypeErrorKind};

/// The whole numbers one type holds, which is what a literal written at it is held to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bounds {
    pub(crate) lowest: i64,
    pub(crate) highest: i64,
}

impl Bounds {
    /// The whole numbers an `Int` holds, which is every one the lexer reads.
    pub(crate) const INT: Self = Self {
        lowest: i64::MIN,
        highest: i64::MAX,
    };

    /// Whether `value` is one of the whole numbers these bounds hold.
    pub(crate) const fn holds(self, value: i64) -> bool {
        self.lowest <= value && value <= self.highest
    }
}

/// The bounds `declaration` states, read off the bodies of its `lowest` and its `highest`.
///
/// # Errors
///
/// Returns the bound whose body is anything other than one whole number.
pub(crate) fn stated(declaration: &InstanceDeclaration) -> Result<Bounds, TypeError> {
    Ok(Bounds {
        lowest: whole_number(declaration, prelude::LOWEST)?,
        highest: whole_number(declaration, prelude::HIGHEST)?,
    })
}

/// The one whole number the method `named` is written as, in the instance that writes it.
fn whole_number(declaration: &InstanceDeclaration, named: &str) -> Result<i64, TypeError> {
    let method = declaration
        .methods
        .iter()
        .find(|method| method.name.text == named)
        .expect("name resolution refused an instance that writes other than its trait's methods");
    read(method).ok_or_else(|| {
        let kind = TypeErrorKind::BoundIsNotAWholeNumber {
            bound: named.to_owned(),
            of: declaration.for_type.text.clone(),
        };
        TypeError::at(method.body.span, kind)
    })
}

/// The whole number `method` is written as, or nothing where its body says anything else.
///
/// A `-` in front of a number is part of the number the parser read, so nothing here looks for
/// one: `-5` and `- 5` are both the one literal `-5` by the time any of this is reached.
fn read(method: &Function) -> Option<i64> {
    let [statement] = method.body.statements.as_slice() else {
        return None;
    };
    let StatementKind::Expr(written) = &statement.kind else {
        return None;
    };
    match written.kind {
        ExprKind::Integer(value) => Some(value),
        _ => None,
    }
}
