//! A pattern as the check sees it, and the values a set of them leaves uncovered.

use std::fmt;

use lumen_ast::{Path, Pattern, PatternKind};
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

    /// Everything `pattern` matches, which an or-pattern matches more than one of.
    ///
    /// An alternative is a whole pattern and may be written inside a constructor, so what an arm
    /// matches is every way of choosing one alternative at each place one is written.
    /// `docs/specs/patterns.md` states that `A | B` in one arm covers what two arms cover.
    pub(crate) fn every(&self, pattern: &Pattern) -> Vec<Pat> {
        match &pattern.kind {
            PatternKind::Integer(_) | PatternKind::String(_) => vec![Pat::Literal],
            PatternKind::Bool(held) => bool_name(*held),
            PatternKind::Wildcard => vec![Pat::Wildcard],
            PatternKind::Name(path) if self.binds(path) => vec![Pat::Wildcard],
            PatternKind::Tuple { path, elements } => self.constructed(&path.to_string(), elements),
            PatternKind::Name(path) | PatternKind::Record { path, .. } => {
                self.constructed(&path.to_string(), &[])
            }
            PatternKind::Or(alternatives) => alternatives
                .iter()
                .flat_map(|alternative| self.every(alternative))
                .collect(),
        }
    }

    /// A constructor pattern carries what its declaration says, whatever the pattern wrote.
    ///
    /// A record pattern names only the fields it binds, and each one binds, so the fields a
    /// declaration lists and the patterns written under it need not line up.
    fn constructed(&self, name: &str, written: &[Pattern]) -> Vec<Pat> {
        let mut carried: Vec<Vec<Pat>> = vec![Vec::new()];
        for position in 0..self.space.carried_by(name) {
            let each = written
                .get(position)
                .map_or_else(|| vec![Pat::Wildcard], |at| self.every(at));
            carried = each_followed_by(&carried, &each);
        }
        carried
            .into_iter()
            .map(|arguments| Pat::Constructed {
                name: name.to_owned(),
                arguments,
            })
            .collect()
    }

    /// Where the module declares each constructor, which is what an arm is placed against.
    pub(crate) const fn space(&self) -> &Space {
        self.space
    }

    /// Whether the name binds rather than naming a constructor, which resolution has decided.
    ///
    /// One reached through a module never binds: a binding is a name of this module, and a name
    /// after a dot is a name of another, which `docs/specs/modules.md` states.
    pub(crate) fn binds(&self, path: &Path) -> bool {
        path.module.is_none()
            && self
                .resolved
                .definition(Namespace::Value, &path.name)
                .is_none_or(|definition| definition.kind != DefinitionKind::Constructor)
    }
}

fn bool_name(held: bool) -> Vec<Pat> {
    let name = if held { "true" } else { "false" };
    vec![Pat::Constructed {
        name: name.to_owned(),
        arguments: Vec::new(),
    }]
}

/// Every way of writing one of `so_far` and then one of `each`, which is what an alternative is.
fn each_followed_by(so_far: &[Vec<Pat>], each: &[Pat]) -> Vec<Vec<Pat>> {
    let mut grown = Vec::new();
    for carried in so_far {
        for one in each {
            let mut row = carried.clone();
            row.push(one.clone());
            grown.push(row);
        }
    }
    grown
}
