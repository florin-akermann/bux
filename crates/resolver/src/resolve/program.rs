//! What the walk yields: a program whose every name points at the definition it means.

use lumen_ast::{Expr, ExprKind, Name, Program, Span};

use super::Definitions;
use crate::definition::{Definition, DefinitionKind, Namespace};

/// A program whose every name points at the definition it means.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedProgram {
    /// The name this module is reached by, which says what the compiler holds for it.
    module: String,
    program: Program,
    definitions: Definitions,
}

impl ResolvedProgram {
    /// The program `module` names, with the definition of every name in it.
    pub(super) const fn new(module: String, program: Program, definitions: Definitions) -> Self {
        Self {
            module,
            program,
            definitions,
        }
    }

    /// The tree this was resolved from.
    #[must_use]
    pub const fn program(&self) -> &Program {
        &self.program
    }

    /// The name of the module this is, which is the name a file or the library gives it.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The module a `.` reaches inside, where the name written before the dot names one.
    ///
    /// The name before the dot says what the dot does, which `docs/specs/calls.md` states, and
    /// every phase after this one has to read it the same way: a module is reached inside, and
    /// anything else is passed in front of the name or read as a field. One answer here is what
    /// keeps them agreeing, because two that drift call a different function and say nothing.
    #[must_use]
    pub fn module_reached<'w>(&self, receiver: &'w Expr) -> Option<&'w Name> {
        let ExprKind::Name(module) = &receiver.kind else {
            return None;
        };
        let definition = self.definition(Namespace::Value, module)?;
        (definition.kind == DefinitionKind::Module).then_some(module)
    }

    /// What `name` means where it is written, which is in one namespace or the other.
    ///
    /// A field label and a name reached through a `.` have none, because neither is a name in
    /// scope; `docs/specs/modules.md` says why.
    #[must_use]
    pub fn definition(&self, namespace: Namespace, name: &Name) -> Option<Definition> {
        self.definitions.get(&(namespace, name.span)).copied()
    }

    /// Every name that has a definition: its namespace, its span, and what it means.
    ///
    /// The order is no order, so a reader that prints them sorts them first. The harness of the
    /// resolver written in Bux prints them in the form `docs/specs/modules.md` states.
    pub fn resolutions(&self) -> impl Iterator<Item = (Namespace, Span, Definition)> + '_ {
        self.definitions
            .iter()
            .map(|(&(namespace, span), &definition)| (namespace, span, definition))
    }
}
