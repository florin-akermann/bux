//! Patterns: what a `match` arm asks of what it matches, and what it binds.

use lumen_ast::{Name, Pattern, PatternKind};
use lumen_resolver::DefinitionKind;

use crate::error::TypeError;
use crate::infer::{Inference, labelled, miscounted};
use crate::scheme::Scheme;
use crate::types::Type;

impl Inference<'_> {
    pub(crate) fn pattern(&mut self, pattern: &Pattern, expected: &Type) -> Result<(), TypeError> {
        match &pattern.kind {
            PatternKind::Integer(_) => self.expect(expected, &Type::int(), pattern.span),
            PatternKind::String(_) => self.expect(expected, &Type::string(), pattern.span),
            PatternKind::Bool(_) => self.expect(expected, &Type::boolean(), pattern.span),
            PatternKind::Name(name) => self.bare_pattern(name, expected),
            PatternKind::Tuple { name, elements } => self.tuple_pattern(name, elements, expected),
            PatternKind::Record { name, fields } => self.record_pattern(name, fields, expected),
        }
    }

    /// A bare name matches a variant that carries nothing, or binds whatever is matched.
    fn bare_pattern(&mut self, name: &Name, expected: &Type) -> Result<(), TypeError> {
        if self.definition_kind(name) == DefinitionKind::Constructor {
            let built = self.value(name);
            return self.expect(expected, &built, name.span);
        }
        self.introduce(name, Scheme::monomorphic(expected.clone()));
        Ok(())
    }

    fn tuple_pattern(
        &mut self,
        name: &Name,
        elements: &[Pattern],
        expected: &Type,
    ) -> Result<(), TypeError> {
        let Type::Function { parameters, result } = self.value(name) else {
            return Err(miscounted(name, 0, elements.len()));
        };
        if parameters.len() != elements.len() {
            return Err(miscounted(name, parameters.len(), elements.len()));
        }
        self.expect(expected, &result, name.span)?;
        for (wanted, element) in parameters.iter().zip(elements) {
            self.pattern(element, &wanted.clone())?;
        }
        Ok(())
    }

    fn record_pattern(
        &mut self,
        name: &Name,
        fields: &[Name],
        expected: &Type,
    ) -> Result<(), TypeError> {
        let key = self.key_of(name);
        let labels = self.environment.labels(&key).to_vec();
        let Type::Function { parameters, result } = self.value(name) else {
            return Err(miscounted(name, 0, fields.len()));
        };
        self.expect(expected, &result, name.span)?;
        for field in fields {
            let index = labelled(&labels, field, &result)?;
            let bound = parameters[index].clone();
            self.introduce(field, Scheme::monomorphic(bound));
        }
        Ok(())
    }
}
