//! How a call passes its arguments, held to what the declaration says about naming.
//!
//! `docs/specs/arguments.md` states the rule: a call names its arguments when the declaration
//! gives two of its parameters one type, and wherever it names them it names them in the order
//! the declaration lists them. A signature the author left unwritten counts exactly as one they
//! wrote out, so a call that names none of its arguments is held to the rule once the whole module
//! is walked: a function declared above its caller is walked after it, in mutual recursion.

use std::mem;

use lumen_ast::{Arguments, Expr, ExprKind, Function, Name, NamedArgument, Span};
use lumen_resolver::{DefinitionKind, Namespace, Origin};

use crate::environment::Key;
use crate::error::{TypeError, TypeErrorKind};
use crate::infer::{Inference, name_of};
use crate::scheme::Scheme;
use crate::types::Type;

/// A call that passes its arguments in order to a function this module declares.
///
/// Whether it had to name them rests on the types inference settles for that function, which the
/// walk may not have reached yet, so the call waits for the whole module.
pub(crate) struct Positional {
    function: Name,
    at: Span,
}

impl Inference<'_> {
    /// One call held to the rule.
    ///
    /// A constructor and a prelude function are neither of them declared as functions of this
    /// module, so neither has parameter names for a call to write or to be held to. A call of
    /// one passes its values in order, and one that names them is `L0411` rather than a call
    /// whose names read as a promise nothing here can keep.
    ///
    /// A call written with its first argument in front is held to the rule before it gets here,
    /// by [`Self::names_none_in_front`].
    pub(crate) fn named_as_declared(
        &mut self,
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
                let function = declared.name.clone();
                self.positional.push(Positional { function, at });
                Ok(())
            }
            (None, Arguments::Named(_)) => Err(TypeError::at(at, self.unnameable(callee, reached))),
            (None, Arguments::Positional(_)) => Ok(()),
        }
    }

    /// A call written with its first argument in front names none of its arguments.
    ///
    /// The receiver is an argument and a dot is no place to write a name, so naming the rest
    /// would name some of the arguments and not others. `docs/specs/calls.md` states it as
    /// `L0423`, and a call earns it ahead of the count: the receiver is one of the arguments
    /// the count counts, so a call that names them reads as one argument too many otherwise.
    pub(crate) fn names_none_in_front(
        &self,
        callee: &Expr,
        arguments: &Arguments,
        at: Span,
    ) -> Result<(), TypeError> {
        let Some(reached) = name_of(callee) else {
            return Ok(());
        };
        if self.in_front(callee).is_none() || matches!(arguments, Arguments::Positional(_)) {
            return Ok(());
        }
        Err(TypeError::at(
            at,
            TypeErrorKind::NamedInFront(reached.text.clone()),
        ))
    }

    /// Why what `written` names has no parameter names for a call to write.
    ///
    /// A name reached through a module is neither: what a module offers is the type of each of
    /// its functions and of each of its constructors, and a type holds no parameter name.
    fn unnameable(&self, callee: &Expr, written: &Name) -> TypeErrorKind {
        if let Some(reached) = self.through_a_module(callee) {
            return TypeErrorKind::NamedThroughModule(reached);
        }
        let built = self
            .resolved
            .definition(Namespace::Value, written)
            .is_some_and(|definition| definition.kind == DefinitionKind::Constructor);
        if built {
            TypeErrorKind::NamedConstructor(written.text.clone())
        } else {
            TypeErrorKind::NamedPrelude(written.text.clone())
        }
    }

    /// The name a call writes where it reaches one through a module, which is `demo.helper`.
    fn through_a_module(&self, callee: &Expr) -> Option<String> {
        let ExprKind::Field { receiver, name } = &callee.kind else {
            return None;
        };
        let module = self.resolved.module_reached(receiver)?;
        Some(format!("{}.{}", module.text, name.text))
    }

    /// Every call of this module that names none of its arguments, held to the rule.
    ///
    /// Every function is walked by now, so each call reads the types inference settled for what
    /// it calls, whether that function is declared above the call or below it.
    ///
    /// # Errors
    ///
    /// Returns the first call that passes two arguments of one type in order.
    pub(crate) fn settle_positional(&mut self) -> Result<(), TypeError> {
        for call in mem::take(&mut self.positional) {
            held_apart(&call.function, &self.takes(&call.function), call.at)?;
        }
        Ok(())
    }

    /// The types the declaration gives its parameters, rather than the ones this call gave them.
    ///
    /// The rule is about the signature, so it reads the same at every call site and names the
    /// type the declaration writes: `T` where a type parameter is repeated, not whatever the
    /// call happened to instantiate it as.
    ///
    /// The environment holds the signature of every function this module declares from before
    /// the first body is walked. A name it holds no function type for takes no parameter at all,
    /// so there is nothing for a call of it to hold apart.
    fn takes(&self, function: &Name) -> Vec<Type> {
        let signature = self
            .environment
            .scheme(&Key::at(function))
            .map(Scheme::body);
        let Some(Type::Function { parameters, .. }) = signature else {
            return Vec::new();
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
