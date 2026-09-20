//! Records: the two things a name followed by braces can mean.

use lumen_ast::{FieldValue, Path, Span};
use lumen_resolver::DefinitionKind;

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::settle::{Lookup, Reached};
use crate::infer::{Inference, labelled, miscounted};
use crate::types::Type;

impl Inference<'_> {
    /// `User { … }` builds a record and `user { … }` updates one; the base says which.
    ///
    /// One reached through a module always builds: an update names a binding, and a binding is
    /// a name of this module, so `demo.User { … }` is the build `docs/specs/modules.md` states.
    pub(crate) fn record(
        &mut self,
        base: &Path,
        fields: &[FieldValue],
        at: Span,
    ) -> Result<Type, TypeError> {
        written_once(base, fields)?;
        if base.module.is_some() || self.definition_kind(&base.name) == DefinitionKind::Constructor
        {
            return self.build(base, fields, at);
        }
        let updated = self.value(&base.name);
        for field in fields {
            let found = self.expr(&field.value)?;
            self.lookups.push(Lookup {
                through: updated.clone(),
                field: field.name.clone(),
                found,
                how: Reached::AsAValue,
            });
        }
        Ok(updated)
    }

    fn build(&mut self, base: &Path, fields: &[FieldValue], at: Span) -> Result<Type, TypeError> {
        let (key, built_as) = self.built_by(base)?;
        let labels = self.environment.labels(&key).to_vec();
        let built = built_as.clone();
        let Type::Function { parameters, result } = built else {
            return build_without_fields(built_as, fields);
        };
        if labels.len() != parameters.len() {
            return Err(built_without_labels(
                base,
                fields,
                parameters.len(),
                &result,
            ));
        }
        for field in fields {
            let found = self.expr(&field.value)?;
            let index = labelled(&labels, &field.name, &result)?;
            self.expect(&parameters[index].clone(), &found, field.value.span)?;
        }
        for label in &labels {
            if !fields.iter().any(|field| field.name.text == *label) {
                let kind = TypeErrorKind::MissingField {
                    of: base.to_string(),
                    field: label.clone(),
                };
                return Err(TypeError::at(at, kind));
            }
        }
        Ok(*result)
    }
}

/// A variant that carries nothing has no field to give a value to.
fn build_without_fields(built: Type, fields: &[FieldValue]) -> Result<Type, TypeError> {
    let Some(field) = fields.first() else {
        return Ok(built);
    };
    let kind = TypeErrorKind::UnknownField {
        of: built,
        field: field.name.text.clone(),
    };
    Err(TypeError::at(field.name.span, kind))
}

/// Reports the first field of `fields` that a field before it already gave a value.
fn written_once(base: &Path, fields: &[FieldValue]) -> Result<(), TypeError> {
    let mut given: Vec<&str> = Vec::new();
    for field in fields {
        if given.contains(&field.name.text.as_str()) {
            let kind = TypeErrorKind::FieldWrittenTwice {
                of: base.to_string(),
                field: field.name.text.clone(),
            };
            return Err(TypeError::at(field.name.span, kind));
        }
        given.push(&field.name.text);
    }
    Ok(())
}

/// A variant that carries its values in order has no field names to write them against.
///
/// Every field written against one is a field it does not have; writing none of them is a call
/// that passed nothing where it carries something.
fn built_without_labels(
    base: &Path,
    fields: &[FieldValue],
    carries: usize,
    built: &Type,
) -> TypeError {
    let Some(field) = fields.first() else {
        return miscounted(&base.name, carries, 0);
    };
    let kind = TypeErrorKind::UnknownField {
        of: built.clone(),
        field: field.name.text.clone(),
    };
    TypeError::at(field.name.span, kind)
}
