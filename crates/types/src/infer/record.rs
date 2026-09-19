//! Records: the two things a name followed by braces can mean.

use lumen_ast::{FieldValue, Name, Span};
use lumen_resolver::DefinitionKind;

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::{Inference, Lookup, labelled, miscounted};
use crate::types::Type;

impl Inference<'_> {
    /// `User { … }` builds a record and `user { … }` updates one; the base says which.
    pub(crate) fn record(
        &mut self,
        base: &Name,
        fields: &[FieldValue],
        at: Span,
    ) -> Result<Type, TypeError> {
        written_once(base, fields)?;
        if self.definition_kind(base) == DefinitionKind::Constructor {
            return self.build(base, fields, at);
        }
        let updated = self.value(base);
        for field in fields {
            let found = self.expr(&field.value)?;
            self.lookups.push(Lookup {
                through: updated.clone(),
                field: field.name.clone(),
                found,
            });
        }
        Ok(updated)
    }

    fn build(&mut self, base: &Name, fields: &[FieldValue], at: Span) -> Result<Type, TypeError> {
        let key = self.key_of(base);
        let labels = self.environment.labels(&key).to_vec();
        let Type::Function { parameters, result } = self.value(base) else {
            return self.build_without_fields(base, fields);
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
                    of: base.text.clone(),
                    field: label.clone(),
                };
                return Err(TypeError::at(at, kind));
            }
        }
        Ok(*result)
    }

    /// A variant that carries nothing has no field to give a value to.
    fn build_without_fields(
        &mut self,
        base: &Name,
        fields: &[FieldValue],
    ) -> Result<Type, TypeError> {
        let built = self.value(base);
        let Some(field) = fields.first() else {
            return Ok(built);
        };
        let kind = TypeErrorKind::UnknownField {
            of: built,
            field: field.name.text.clone(),
        };
        Err(TypeError::at(field.name.span, kind))
    }
}

/// Reports the first field of `fields` that a field before it already gave a value.
fn written_once(base: &Name, fields: &[FieldValue]) -> Result<(), TypeError> {
    let mut given: Vec<&str> = Vec::new();
    for field in fields {
        if given.contains(&field.name.text.as_str()) {
            let kind = TypeErrorKind::FieldWrittenTwice {
                of: base.text.clone(),
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
    base: &Name,
    fields: &[FieldValue],
    carries: usize,
    built: &Type,
) -> TypeError {
    let Some(field) = fields.first() else {
        return miscounted(base, carries, 0);
    };
    let kind = TypeErrorKind::UnknownField {
        of: built.clone(),
        field: field.name.text.clone(),
    };
    TypeError::at(field.name.span, kind)
}
