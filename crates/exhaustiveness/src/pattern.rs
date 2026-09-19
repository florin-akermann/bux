//! A pattern as the check sees it, and the values a set of them leaves uncovered.

use std::fmt;

use lumen_ast::{Name, Pattern, PatternKind};
use lumen_resolver::{DefinitionKind, Namespace, ResolvedProgram};

use crate::space::Space;

/// What one pattern matches, with everything inference has already settled left out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Pat {
    /// A name that binds, which matches every value of its type.
    Wildcard,
    /// A constructor, and what it matches of the values that constructor carries.
    Constructed { name: String, arguments: Vec<Pat> },
    /// A number or a quoted string.
    ///
    /// Which one it is never changes the answer: a type with more values than a `match` can list
    /// is covered by a name that binds and by nothing else.
    Literal,
}

impl fmt::Display for Pat {
    /// A pattern as an arm would write it, which is how an uncovered value is named.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Constructed { name, arguments } = self else {
            return f.write_str("_");
        };
        f.write_str(name)?;
        if arguments.is_empty() {
            return Ok(());
        }
        let written: Vec<String> = arguments.iter().map(ToString::to_string).collect();
        write!(f, "({})", written.join(", "))
    }
}

/// Patterns as written, read against what the module declares.
pub(crate) struct Reading<'a> {
    resolved: &'a ResolvedProgram,
    space: &'a Space,
}

impl<'a> Reading<'a> {
    pub(crate) const fn new(resolved: &'a ResolvedProgram, space: &'a Space) -> Self {
        Self { resolved, space }
    }

    /// What `pattern` matches.
    pub(crate) fn of(&self, pattern: &Pattern) -> Pat {
        match &pattern.kind {
            PatternKind::Integer(_) | PatternKind::String(_) => Pat::Literal,
            PatternKind::Bool(held) => self.constructed(&bool_name(*held), &[]),
            PatternKind::Name(name) if self.binds(name) => Pat::Wildcard,
            PatternKind::Tuple { name, elements } => self.constructed(&name.text, elements),
            PatternKind::Name(name) | PatternKind::Record { name, .. } => {
                self.constructed(&name.text, &[])
            }
        }
    }

    /// A constructor pattern carries what its declaration says, whatever the pattern wrote.
    ///
    /// A record pattern names only the fields it binds, and each one binds, so the fields a
    /// declaration lists and the patterns written under it need not line up.
    fn constructed(&self, name: &str, written: &[Pattern]) -> Pat {
        let arguments = (0..self.space.carried_by(name))
            .map(|carried| written.get(carried).map_or(Pat::Wildcard, |at| self.of(at)))
            .collect();
        Pat::Constructed {
            name: name.to_owned(),
            arguments,
        }
    }

    /// Whether the name binds rather than naming a constructor, which resolution has decided.
    fn binds(&self, name: &Name) -> bool {
        self.resolved
            .definition(Namespace::Value, name)
            .is_none_or(|definition| definition.kind != DefinitionKind::Constructor)
    }
}

fn bool_name(held: bool) -> String {
    if held { "true" } else { "false" }.to_owned()
}
