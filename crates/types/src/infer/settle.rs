//! What a function settles once its body has been walked.
//!
//! A field waits on the type it is reached through, and an addition and a comparison wait on
//! the type of what they are given. Each of them is answered when the function they are
//! written in has been inferred, so the report lands where the source wrote it.

use std::mem;

use lumen_ast::Name;

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::{Inference, labelled};
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
        for (added, at) in mem::take(&mut self.additions) {
            let found = self.table.shallow(&added);
            if matches!(found, Type::Var(_)) {
                self.expect(&Type::int(), &added, at)?;
            } else if found != Type::int() && found != Type::string() {
                let kind = TypeErrorKind::NotAddable(self.table.solved(&found));
                return Err(TypeError::at(at, kind));
            }
        }
        Ok(())
    }

    /// The comparisons of one function, each between two values of a type that has `Eq`.
    ///
    /// Version 0.1 has no `derive`, so the types that have `Eq` are the three the library ships:
    /// `Int`, `Bool`, and `String`. A comparison still untyped once its function is inferred is
    /// a comparison of `Int`s, as an addition still untyped is an addition of them.
    pub(crate) fn settle_equalities(&mut self) -> Result<(), TypeError> {
        for (compared, at) in mem::take(&mut self.equalities) {
            let found = self.table.shallow(&compared);
            if matches!(found, Type::Var(_)) {
                self.expect(&Type::int(), &compared, at)?;
            } else if !has_eq(&found) {
                let kind = TypeErrorKind::NotEquatable(self.table.solved(&found));
                return Err(TypeError::at(at, kind));
            }
        }
        Ok(())
    }

    pub(crate) fn look_up(&mut self, lookup: Lookup) -> Result<(), TypeError> {
        let through = self.table.solved(&lookup.through);
        let field = lookup.field;
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
}

/// Whether `found` has `Eq`, which in version 0.1 the three types the library ships do.
fn has_eq(found: &Type) -> bool {
    [Type::int(), Type::boolean(), Type::string()].contains(found)
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
