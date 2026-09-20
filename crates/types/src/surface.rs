//! What a module offers the modules that import it, and what they offer back.
//!
//! `docs/specs/modules.md` states the rule this holds: what a loaded module offers is every
//! function it declares, and a type it declares stays its own.

use std::collections::{HashMap, HashSet};

use lumen_ast::Item;
use lumen_resolver::ResolvedProgram;

use crate::scheme::Scheme;
use crate::types::Type;

/// Every function a module declares, and the types it keeps to itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Surface {
    functions: HashMap<String, Offered>,
    declared: HashSet<String>,
}

/// What one function of a module amounts to for a module that imports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Offered {
    /// A function written once, whose type is fresh at each use.
    Plain(Scheme),
    /// A function written once per set of types it is used at, which `docs/specs/codegen.md`
    /// states, so only the module declaring it knows which of them to write.
    Generic,
}

impl Offered {
    /// What `scheme` amounts to, which is what it is polymorphic over rather than what was
    /// written.
    ///
    /// A function that writes no type parameter is generic all the same when nothing settled
    /// its type: inference generalises over whatever it left free, and the class the declaring
    /// module writes is erased exactly as a written parameter erases it.
    fn of(scheme: Scheme) -> Self {
        if scheme.quantified().is_empty() {
            Self::Plain(scheme)
        } else {
            Self::Generic
        }
    }
}

impl Surface {
    /// What `resolved` offers, where `functions` is the type inference gave each of them.
    pub(crate) fn of(resolved: &ResolvedProgram, functions: HashMap<String, Scheme>) -> Self {
        let declared = resolved
            .program()
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Type(declaration) => Some(declaration.name.text.clone()),
                Item::Import(_) | Item::Trait(_) | Item::Instance(_) | Item::Function(_) => None,
            })
            .collect();
        let offered = functions
            .into_iter()
            .map(|(name, scheme)| (name, Offered::of(scheme)))
            .collect();
        Self {
            functions: offered,
            declared,
        }
    }

    /// What the module offers under `name`, when it declares a function of that name.
    pub(crate) fn function(&self, name: &str) -> Option<&Offered> {
        self.functions.get(name)
    }

    /// The type this module keeps to itself that `scheme` names, when it names one.
    ///
    /// A type is written as a bare name, so a module importing this one has no way to write it,
    /// and a function whose signature names one offers nothing that can be taken up.
    pub(crate) fn kept_to_itself(&self, scheme: &Scheme) -> Option<&str> {
        self.named_by(scheme.body())
    }

    fn named_by<'a>(&'a self, written: &Type) -> Option<&'a str> {
        match written {
            Type::Named { name, arguments } => self
                .declared
                .get(name.as_str())
                .map(String::as_str)
                .or_else(|| {
                    arguments
                        .iter()
                        .find_map(|argument| self.named_by(argument))
                }),
            Type::Function { parameters, result } => parameters
                .iter()
                .find_map(|parameter| self.named_by(parameter))
                .or_else(|| self.named_by(result)),
            Type::Var(_) | Type::Parameter(_) | Type::Module(_) | Type::Unit => None,
        }
    }
}

/// What the modules a module imports offer it, under the names it imports them by.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Imported {
    surfaces: HashMap<String, Surface>,
}

impl Imported {
    /// The same, with `module` now offering `surface` to whatever imports it.
    #[must_use]
    pub fn offering(mut self, module: &str, surface: Surface) -> Self {
        self.surfaces.insert(module.to_owned(), surface);
        self
    }

    /// What `module` offers, when it is one of the modules loaded.
    pub(crate) fn surface(&self, module: &str) -> Option<&Surface> {
        self.surfaces.get(module)
    }
}
