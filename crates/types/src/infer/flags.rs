//! Which parameters a declaration may take, which is every type but a bare `Bool`.
//!
//! `docs/specs/arguments.md` states the rule: `open(true)` says nothing, so a two-variant type
//! takes the flag's place and the call says which of the two it means. The one carve-out is a
//! boolean operation, whose parameters are `Bool` because that is what the function is about.
//!
//! The rule is about the types an author chose, so it reaches an instance method through the
//! trait rather than at the instance: `hashed(value: Bool) -> Int` is `Hash<Bool>` doing what
//! `trait Hash<T>` said, and the `T` is where a flag would have had to be written.
//! A trait's own signature is therefore held to the rule, and an instance's is not.
//! `docs/design.md` states that scope.
//!
//! The types read are the ones inference settled, as the naming rule reads them, so the check
//! runs once the function it is about has been walked.

use lumen_ast::{Function, Name, Parameter, Signature};

use crate::environment::Key;
use crate::error::{TypeError, TypeErrorKind};
use crate::infer::Inference;
use crate::scheme::Scheme;
use crate::types::Type;

impl Inference<'_> {
    /// The parameters of one function, each of them something a call can say more than `true` of.
    ///
    /// An instance method has none to answer for: its trait wrote them.
    ///
    /// # Errors
    ///
    /// Returns the first parameter that is a bare `Bool` outside a boolean operation.
    pub(crate) fn settle_parameters(
        &mut self,
        function: &Function,
        key: &Key,
    ) -> Result<(), TypeError> {
        if self.environment.written_as(key).is_some() {
            return Ok(());
        }
        let taken = self.parameter_types(function);
        if operates_on_booleans(&taken, &self.table.solved(&self.result)) {
            return Ok(());
        }
        for (written, found) in function.parameters.iter().zip(&taken) {
            if *found == Type::boolean() {
                return Err(flag(written, &function.name));
            }
        }
        Ok(())
    }

    /// The parameters of one method a trait declares, held to the rule a function's are held to.
    ///
    /// This is where the rule reaches an instance method's types, because this is where they were
    /// chosen: `instance Hash<Bool>` writes what `trait Hash<T>` wrote, and a `Bool` the trait
    /// wrote outright is a flag every instance of it would have to take.
    ///
    /// A trait's signature is not a body, so nothing infers it and this reads the type the
    /// environment gave it, exactly as the naming rule does.
    ///
    /// # Errors
    ///
    /// Returns the first parameter that is a bare `Bool` outside a boolean operation.
    pub(crate) fn settle_method_parameters(&self, method: &Signature) -> Result<(), TypeError> {
        let key = Key::at(&method.name);
        let Some(Type::Function { parameters, result }) =
            self.environment.scheme(&key).map(Scheme::body)
        else {
            unreachable!("a trait gave each of its methods a function type")
        };
        if operates_on_booleans(parameters, result) {
            return Ok(());
        }
        for (written, found) in method.parameters.iter().zip(parameters) {
            if *found == Type::boolean() {
                return Err(flag(written, &method.name));
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

/// A parameter a call of `named` could say no more than `true` of.
fn flag(written: &Parameter, named: &Name) -> TypeError {
    let kind = TypeErrorKind::FlagParameter(named.text.clone());
    TypeError::at(written.span, kind)
}
