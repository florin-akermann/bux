//! Which parameters a declaration may take, which is every type but a bare `Bool`.
//!
//! `docs/specs/arguments.md` states the rule: `open(true)` says nothing, so a two-variant type
//! takes the flag's place and the call says which of the two it means. The one carve-out is a
//! boolean operation, whose parameters are `Bool` because that is what the function is about.
//!
//! The types read are the ones inference settled, as the naming rule reads them, so the check
//! runs once the function it is about has been walked.

use lumen_ast::{Function, Parameter};

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::Inference;
use crate::types::Type;

impl Inference<'_> {
    /// The parameters of one function, each of them something a call can say more than `true` of.
    ///
    /// # Errors
    ///
    /// Returns the first parameter that is a bare `Bool` outside a boolean operation.
    pub(crate) fn settle_parameters(&mut self, function: &Function) -> Result<(), TypeError> {
        let taken = self.parameter_types(function);
        if operates_on_booleans(&taken, &self.table.solved(&self.result)) {
            return Ok(());
        }
        for (written, found) in function.parameters.iter().zip(&taken) {
            if *found == Type::boolean() {
                return Err(flag(written, function));
            }
        }
        Ok(())
    }

    /// The type inference settled for each parameter, in the order the declaration lists them.
    fn parameter_types(&self, function: &Function) -> Vec<Type> {
        function
            .parameters
            .iter()
            .map(|written| self.parameter_type(written))
            .collect()
    }

    /// The type of one parameter, which it was given a name for before the body was walked.
    fn parameter_type(&self, written: &Parameter) -> Type {
        let found = self
            .types
            .get(&written.name.span)
            .unwrap_or_else(|| unreachable!("a parameter is typed before the body is walked"));
        self.table.solved(found)
    }
}

/// Whether a signature is a boolean operation, which is the one carve-out the rule has.
///
/// `Bool` is what such a function is about rather than something it is told, so a parameter of it
/// is an operand. A `Bool` among other types, or one whose function gives back something else, is
/// a choice the call makes and says nothing about.
fn operates_on_booleans(taken: &[Type], gives: &Type) -> bool {
    *gives == Type::boolean() && taken.iter().all(|one| *one == Type::boolean())
}

/// A parameter a call of `function` could say no more than `true` of.
fn flag(written: &Parameter, function: &Function) -> TypeError {
    let kind = TypeErrorKind::FlagParameter(function.name.text.clone());
    TypeError::at(written.span, kind)
}
