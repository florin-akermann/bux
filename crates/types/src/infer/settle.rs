//! What a function settles once its body has been walked.
//!
//! A field waits on the type it is reached through, and an addition and a comparison wait on
//! the type of what they are given. Each of them is answered when the function they are
//! written in has been inferred, so the report lands where the source wrote it.

use std::mem;

use lumen_ast::{Name, Span};

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::{Inference, labelled};
use crate::supplied;
use crate::types::Type;

impl Inference<'_> {
    /// The fields of one function, now that its body has said what they are reached through.
    ///
    /// They settle before the body meets the declared result, so a field that disagrees with what
    /// its record declares is reported where it is written rather than as a body of the wrong type.
    pub(crate) fn look_up_fields(&mut self) -> Result<(), TypeError> {
        for lookup in mem::take(&mut self.lookups) {
            self.look_up(lookup)?;
        }
        Ok(())
    }

    /// The additions of one function, each an addition of `Int`s unless something said otherwise.
    ///
    /// They settle last, after the declared result has had its say, because the result is often
    /// the only thing that says an addition joins two `String`s.
    pub(crate) fn settle_additions(&mut self) -> Result<(), TypeError> {
        let waiting = mem::take(&mut self.additions);
        self.settled(waiting, &Type::int(), not_addable)
    }

    /// The comparisons of one function, each between two values of a type that has `Eq`.
    ///
    /// Version 0.1 has no `derive`, so the types that have `Eq` are the three the library ships:
    /// `Int`, `Bool`, and `String`. A comparison still untyped once its function is inferred is
    /// a comparison of `Int`s, as an addition still untyped is an addition of them.
    pub(crate) fn settle_equalities(&mut self) -> Result<(), TypeError> {
        let waiting = mem::take(&mut self.equalities);
        self.settled(waiting, &Type::int(), not_equatable)
    }

    /// The statements of one function that nothing takes the value of.
    ///
    /// They settle after the comparisons, so a discarded `a == b` is named as the `Bool` it is
    /// rather than as a type nothing had settled yet. A statement inference never settled takes
    /// `()`, as an unsettled addition takes `Int`, so `todo("not yet")` stands as a statement.
    pub(crate) fn settle_discards(&mut self) -> Result<(), TypeError> {
        let waiting = mem::take(&mut self.discards);
        self.settled(waiting, &Type::Unit, not_discardable)
    }

    /// Each type that was waiting on the function it is written in, now that the function is done.
    ///
    /// A type nothing settled takes `default`, which is the one default the language has and the
    /// reason `1 + 1` is an addition of `Int`s. A type that did settle is put to `refused`, which
    /// says what is wrong with it or that nothing is.
    fn settled(
        &mut self,
        waiting: Vec<(Type, Span)>,
        default: &Type,
        refused: fn(&Type) -> Option<TypeErrorKind>,
    ) -> Result<(), TypeError> {
        for (found, at) in waiting {
            let settled = self.table.shallow(&found);
            if matches!(settled, Type::Var(_)) {
                self.expect(default, &found, at)?;
            } else if let Some(kind) = refused(&self.table.solved(&settled)) {
                return Err(TypeError::at(at, kind));
            }
        }
        Ok(())
    }

    pub(crate) fn look_up(&mut self, lookup: Lookup) -> Result<(), TypeError> {
        let through = self.table.solved(&lookup.through);
        let field = lookup.field;
        if let Type::Module(module) = &through {
            return self.inside(module, &field, &lookup.found);
        }
        let Type::Named { name, .. } = &through else {
            return Err(unreachable_field(&through, &field));
        };
        let Some(key) = self.environment.record(name).cloned() else {
            return Err(unreachable_field(&through, &field));
        };
        let labels = self.environment.labels(&key).to_vec();
        let scheme = self.scheme(&key);
        let Type::Function { parameters, result } = scheme.instantiate(&mut self.table) else {
            return Err(unreachable_field(&through, &field));
        };
        self.expect(&through, &result, field.span)?;
        let index = labelled(&labels, &field, &self.table.solved(&result))?;
        self.expect(&lookup.found, &parameters[index].clone(), field.span)
    }

    /// A name reached inside a module, which only a module the compiler supplies has any of.
    fn inside(&mut self, module: &str, field: &Name, found: &Type) -> Result<(), TypeError> {
        if !supplied::supplies(module) {
            return Err(unreachable_field(&Type::Module(module.to_owned()), field));
        }
        let Some(declared) = supplied::declared(module, &field.text) else {
            let kind = TypeErrorKind::NotInSuppliedModule {
                module: module.to_owned(),
                name: field.text.clone(),
            };
            return Err(TypeError::at(field.span, kind));
        };
        self.expect(found, &declared, field.span)
    }
}

/// What is wrong with adding two of `found`, which is that `+` joins `Int`s or `String`s.
fn not_addable(found: &Type) -> Option<TypeErrorKind> {
    let addable = *found == Type::int() || *found == Type::string();
    (!addable).then(|| TypeErrorKind::NotAddable(found.clone()))
}

/// What is wrong with comparing two of `found`, which is that `==` needs `Eq`.
///
/// Version 0.1 has no `derive`, so `Eq` is the library's, on the three types it ships it for.
fn not_equatable(found: &Type) -> Option<TypeErrorKind> {
    let equatable = [Type::int(), Type::boolean(), Type::string()].contains(found);
    (!equatable).then(|| TypeErrorKind::NotEquatable(found.clone()))
}

/// What is wrong with leaving a `found` behind, which is that nothing is there to take it.
fn not_discardable(found: &Type) -> Option<TypeErrorKind> {
    (*found != Type::Unit).then(|| TypeErrorKind::Discarded(found.clone()))
}

/// A field waiting on the type it is reached through.
pub(crate) struct Lookup {
    pub(crate) through: Type,
    pub(crate) field: Name,
    pub(crate) found: Type,
}

/// A field reached through something that has no fields, or through a type nothing settled.
fn unreachable_field(through: &Type, field: &Name) -> TypeError {
    let kind = if let Type::Module(module) = through {
        TypeErrorKind::InModule {
            module: module.clone(),
            name: field.text.clone(),
        }
    } else if matches!(through, Type::Var(_)) {
        TypeErrorKind::UnknownReceiver(field.text.clone())
    } else {
        TypeErrorKind::UnknownField {
            of: through.clone(),
            field: field.text.clone(),
        }
    };
    TypeError::at(field.span, kind)
}
