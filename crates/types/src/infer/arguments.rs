//! How a call passes its arguments, held to what the declaration says about naming.
//!
//! `docs/specs/arguments.md` states the rule: a call names its arguments when the declaration
//! gives two of its parameters one type, and wherever it names them it names them in the order
//! the declaration lists them. A module is inferred bottom up, so a declaration has the types it
//! has by the time anything calls it, and a signature the author left unwritten counts exactly as
//! one they wrote out.

use lumen_ast::{Arguments, Expr, Function, Name, NamedArgument, Span};
use lumen_resolver::{DefinitionKind, Namespace, Origin};

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::{Inference, name_of};
use crate::types::Type;

impl Inference<'_> {
    /// One call held to the rule.
    ///
    /// A constructor and a prelude function are neither of them declared as functions of this
    /// module, so neither has parameter names for a call to write or to be held to. A call of
    /// one passes its values in order, and one that names them is `L0411` rather than a call
    /// whose names read as a promise nothing here can keep.
    pub(crate) fn named_as_declared(
        &self,
        callee: &Expr,
        arguments: &Arguments,
        at: Span,
    ) -> Result<(), TypeError> {
        let Some(reached) = name_of(callee) else {
            return Ok(());
        };
        match (self.declaration(reached), arguments) {
            (Some(declared), Arguments::Named(written)) => in_order(declared, written),
            (Some(declared), Arguments::Positional(_)) => {
                held_apart(&declared.name, &self.takes(declared), at)
            }
            (None, Arguments::Named(_)) => Err(TypeError::at(at, self.unnameable(reached))),
            (None, Arguments::Positional(_)) => Ok(()),
        }
    }

    /// Why what `called` names has no parameter names for a call to write.
    fn unnameable(&self, called: &Name) -> TypeErrorKind {
        let built = self
            .resolved
            .definition(Namespace::Value, called)
            .is_some_and(|definition| definition.kind == DefinitionKind::Constructor);
        if built {
            TypeErrorKind::NamedConstructor(called.text.clone())
        } else {
            TypeErrorKind::NamedPrelude(called.text.clone())
        }
    }

    /// The types the declaration gives its parameters, rather than the ones this call gave them.
    ///
    /// The rule is about the signature, so it reads the same at every call site and names the
    /// type the declaration writes: `T` where a type parameter is repeated, not whatever the
    /// call happened to instantiate it as.
    ///
    /// A function is given its type before its body is walked, and a module reads top down, so
    /// every function a call can reach has one by the time the call is reached.
    fn takes(&self, declared: &Function) -> Vec<Type> {
        let Some(Type::Function { parameters, .. }) = self.types.get(&declared.name.span) else {
            unreachable!("a function has its type before anything below it can call it")
        };
        parameters
            .iter()
            .map(|one| self.table.solved(one))
            .collect()
    }

    /// The function `called` names, where it names one this module declares.
    fn declaration(&self, called: &Name) -> Option<&Function> {
        let definition = self.resolved.definition(Namespace::Value, called)?;
        let Origin::Declared(at) = definition.origin else {
            return None;
        };
        self.resolved
            .program()
            .functions()
            .find(|function| function.name.span == at)
    }
}

/// A call that names none of its arguments, which the types alone must then hold apart.
fn held_apart(function: &Name, parameters: &[Type], at: Span) -> Result<(), TypeError> {
    let repeated = parameters
        .iter()
        .enumerate()
        .find_map(|(index, one)| parameters[..index].contains(one).then_some(one));
    let Some(repeated) = repeated else {
        return Ok(());
    };
    let kind = TypeErrorKind::Unnamed {
        function: function.text.clone(),
        repeated: repeated.clone(),
    };
    Err(TypeError::at(at, kind))
}

/// A call that names its arguments, which it writes in the order the declaration lists them.
fn in_order(declared: &Function, written: &[NamedArgument]) -> Result<(), TypeError> {
    for (argument, parameter) in written.iter().zip(&declared.parameters) {
        if argument.name.text != parameter.name.text {
            let kind = TypeErrorKind::Misnamed {
                written: argument.name.text.clone(),
                parameter: parameter.name.text.clone(),
            };
            return Err(TypeError::at(argument.span(), kind));
        }
    }
    Ok(())
}
