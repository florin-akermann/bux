//! A pattern: what a value has to be, and the names the arm below it goes on to read.
//!
//! `docs/specs/patterns.md` states the forms. Resolution decides only which names bind and
//! which name a constructor, because a bare name is both until this phase says which.

use lumen_ast::{Name, Path, Pattern, PatternKind};

use crate::definition::{Definition, DefinitionKind};
use crate::error::{ResolveError, ResolveErrorKind};
use crate::resolve::{Resolved, Resolver};

impl Resolver {
    pub(super) fn pattern(&mut self, pattern: &Pattern) -> Resolved {
        match &pattern.kind {
            PatternKind::Name(path) => self.bare_pattern(path),
            PatternKind::Tuple { path, elements } => {
                self.built_as(path)?;
                for element in elements {
                    self.pattern(element)?;
                }
                Ok(())
            }
            PatternKind::Record { path, fields } => {
                self.built_as(path)?;
                for field in fields {
                    self.introduce_value(field, DefinitionKind::Local)?;
                }
                Ok(())
            }
            PatternKind::Or(alternatives) => self.any_of(alternatives),
            PatternKind::Integer(_)
            | PatternKind::String(_)
            | PatternKind::Bool(_)
            | PatternKind::Wildcard => Ok(()),
        }
    }

    /// `Pending | Running`, whose alternatives match without binding anything.
    ///
    /// `docs/specs/patterns.md` states why: alternatives that bind would have to bind the same
    /// names at the same types, which is a rule about the set of them rather than about each,
    /// and the arms it would save are written as one arm each.
    fn any_of(&mut self, alternatives: &[Pattern]) -> Resolved {
        for alternative in alternatives {
            if let Some(bound) = self.binds(alternative) {
                let kind = ResolveErrorKind::BindsInsideOr(bound.text.clone());
                return Err(ResolveError::at(bound, kind));
            }
            self.pattern(alternative)?;
        }
        Ok(())
    }

    /// The first name `pattern` would bind, which is nothing where it binds nothing.
    fn binds<'w>(&self, pattern: &'w Pattern) -> Option<&'w Name> {
        match &pattern.kind {
            PatternKind::Name(path) => {
                let bare = path.module.is_none();
                let constructs = self
                    .values
                    .look_up(&path.name.text)
                    .is_some_and(is_constructor);
                (bare && !constructs).then_some(&path.name)
            }
            PatternKind::Record { fields, .. } => fields.first(),
            PatternKind::Tuple { elements, .. } => {
                elements.iter().find_map(|element| self.binds(element))
            }
            PatternKind::Or(alternatives) => alternatives.iter().find_map(|one| self.binds(one)),
            PatternKind::Integer(_)
            | PatternKind::String(_)
            | PatternKind::Bool(_)
            | PatternKind::Wildcard => None,
        }
    }

    /// A bare name matches what a constructor of that name carries, and otherwise binds the value.
    ///
    /// One reached through a module matches and never binds: a binding is a name of this module,
    /// and a dotted name is a name of another, which `docs/specs/modules.md` states.
    fn bare_pattern(&mut self, path: &Path) -> Resolved {
        if path.module.is_some() {
            return self.reached_through(path);
        }
        let name = &path.name;
        if self.values.look_up(&name.text).is_some_and(is_constructor) {
            return self.use_value(name);
        }
        self.introduce_value(name, DefinitionKind::Local)
    }

    /// The name a value is built with, which is this module's constructor or another module's.
    pub(super) fn built_as(&mut self, path: &Path) -> Resolved {
        if path.module.is_some() {
            return self.reached_through(path);
        }
        self.use_value(&path.name)
    }
}

fn is_constructor(definition: Definition) -> bool {
    definition.kind == DefinitionKind::Constructor
}
