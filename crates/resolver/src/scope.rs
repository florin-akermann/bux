//! The names in scope, as layers that names leave when their layer does.

use std::collections::HashMap;

use lumen_ast::Name;

use crate::definition::{Definition, DefinitionKind, Namespace, Origin};

/// One namespace: the types or the values, never both.
///
/// A name is introduced into the innermost layer, and looked up from there outwards. Nothing is
/// ever replaced: a name already in scope is a [`Clash`], which is how `docs/design.md` section
/// 16's one-definition rule is kept.
pub(crate) struct Scope {
    namespace: Namespace,
    layers: Vec<HashMap<String, Definition>>,
}

impl Scope {
    /// A scope holding the prelude's `supplied` names and nothing else.
    pub(crate) fn of_prelude(
        namespace: Namespace,
        supplied: &[&str],
        kind: DefinitionKind,
    ) -> Self {
        let prelude = supplied
            .iter()
            .map(|name| {
                (
                    (*name).to_owned(),
                    Definition {
                        kind,
                        origin: Origin::Prelude,
                    },
                )
            })
            .collect();
        Self {
            namespace,
            layers: vec![prelude],
        }
    }

    /// Which of the two scopes this is, which decides how a name missing from it reads.
    pub(crate) const fn namespace(&self) -> Namespace {
        self.namespace
    }

    pub(crate) fn enter(&mut self) {
        self.layers.push(HashMap::new());
    }

    pub(crate) fn leave(&mut self) {
        self.layers.pop();
    }

    /// Adds `name` to the innermost layer, refusing anything already in scope.
    pub(crate) fn introduce(&mut self, name: &Name, kind: DefinitionKind) -> Result<(), Clash> {
        if self.look_up(&name.text).is_some() {
            return Err(self.clash_over(&name.text));
        }
        let definition = Definition {
            kind,
            origin: Origin::Declared(name.span),
        };
        self.innermost_mut().insert(name.text.clone(), definition);
        Ok(())
    }

    /// The definition `text` means here, when it means one.
    pub(crate) fn look_up(&self, text: &str) -> Option<Definition> {
        self.layers
            .iter()
            .rev()
            .find_map(|layer| layer.get(text))
            .copied()
    }

    /// Whether the name that is already in scope is one this layer declares, or one further out.
    fn clash_over(&self, text: &str) -> Clash {
        if self.innermost().contains_key(text) {
            Clash::Twice
        } else {
            Clash::Shadowed
        }
    }

    fn innermost(&self) -> &HashMap<String, Definition> {
        self.layers.last().expect("a scope always has a layer")
    }

    fn innermost_mut(&mut self) -> &mut HashMap<String, Definition> {
        self.layers.last_mut().expect("a scope always has a layer")
    }
}

/// Why a name could not be introduced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Clash {
    /// The same layer already declares it, so the two are two definitions of one name.
    Twice,
    /// A layer further out declares it, so the new one would hide it.
    Shadowed,
}
