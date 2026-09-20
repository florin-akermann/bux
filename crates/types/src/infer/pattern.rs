//! Patterns: what a `match` arm asks of what it matches, and what it binds.

use lumen_ast::{Name, Path, Pattern, PatternKind, Span};
use lumen_resolver::{DefinitionKind, prelude};

use crate::error::TypeError;
use crate::infer::{Asked, Inference, Requirement, labelled, miscounted};
use crate::scheme::{Required, Scheme};
use crate::types::Type;

impl Inference<'_> {
    pub(crate) fn pattern(&mut self, pattern: &Pattern, expected: &Type) -> Result<(), TypeError> {
        match &pattern.kind {
            PatternKind::Integer(value) => self.number_pattern(*value, expected, pattern.span),
            PatternKind::String(_) => self.literal_pattern(&Type::string(), expected, pattern.span),
            PatternKind::Bool(_) => self.literal_pattern(&Type::boolean(), expected, pattern.span),
            PatternKind::Name(path) => self.bare_pattern(path, expected),
            PatternKind::Tuple { path, elements } => self.tuple_pattern(path, elements, expected),
            PatternKind::Record { path, fields } => self.record_pattern(path, fields, expected),
            PatternKind::Or(alternatives) => self.any_of(alternatives, expected),
            PatternKind::Wildcard => Ok(()),
        }
    }

    /// A whole number, which takes the type it is matched against as one written anywhere does.
    ///
    /// `docs/specs/patterns.md` holds a pattern to `docs/specs/literals.md`: the number settles
    /// on whatever the scrutinee's type is, so `5` over an `Int32` is that type's `5`.
    fn number_pattern(&mut self, value: i64, expected: &Type, at: Span) -> Result<(), TypeError> {
        let written = self.literal(value, at);
        self.literal_pattern(&written, expected, at)
    }

    /// A literal, which matches the one value it is and asks `Eq` whether it is that value.
    ///
    /// The type it is written over is recorded, because lowering reads the instance of `Eq` and
    /// of `IntegerLiteral` off the type at this span, exactly as it reads an operator's.
    fn literal_pattern(
        &mut self,
        written: &Type,
        expected: &Type,
        at: Span,
    ) -> Result<(), TypeError> {
        self.expect(expected, written, at)?;
        self.requirements.push(Requirement {
            required: Required {
                trait_name: prelude::EQ.to_owned(),
                at: expected.clone(),
            },
            written: at,
            how: Asked::Constraint,
        });
        self.types.insert(at, expected.clone());
        Ok(())
    }

    /// `Pending | Running`: every alternative matches the one type, and none of them binds.
    ///
    /// Resolution has already refused an alternative that binds, so nothing here introduces a
    /// name and the arm reads the same whichever alternative answered.
    fn any_of(&mut self, alternatives: &[Pattern], expected: &Type) -> Result<(), TypeError> {
        for alternative in alternatives {
            self.pattern(alternative, expected)?;
        }
        Ok(())
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
