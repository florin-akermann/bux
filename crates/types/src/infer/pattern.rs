//! Patterns: what a `match` arm asks of what it matches, and what it binds.

use lumen_ast::{Name, Path, Pattern, PatternKind};
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
            PatternKind::Name(path) => self.bare_pattern(path, expected),
            PatternKind::Tuple { path, elements } => self.tuple_pattern(path, elements, expected),
            PatternKind::Record { path, fields } => self.record_pattern(path, fields, expected),
        }
    }

    /// A bare name matches a variant that carries nothing, or binds whatever is matched.
    ///
    /// One reached through a module always matches: nothing binds a name of another module.
    fn bare_pattern(&mut self, path: &Path, expected: &Type) -> Result<(), TypeError> {
        let name = &path.name;
        if path.module.is_none() && self.definition_kind(name) != DefinitionKind::Constructor {
            self.introduce(name, Scheme::monomorphic(expected.clone()));
            return Ok(());
        }
        let (_, built) = self.built_by(path)?;
        self.expect(expected, &built, name.span)
    }

    fn tuple_pattern(
        &mut self,
        path: &Path,
        elements: &[Pattern],
        expected: &Type,
    ) -> Result<(), TypeError> {
        let name = &path.name;
        let (_, built) = self.built_by(path)?;
        let Type::Function { parameters, result } = built else {
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
        path: &Path,
        fields: &[Name],
        expected: &Type,
    ) -> Result<(), TypeError> {
        let name = &path.name;
        let (key, built) = self.built_by(path)?;
        let labels = self.environment.labels(&key).to_vec();
        let Type::Function { parameters, result } = built else {
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
