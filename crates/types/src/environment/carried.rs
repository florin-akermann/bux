//! The prelude, read out of the Lumen source the compiler carries.
//!
//! `docs/specs/library.md` says the prelude is `library/prelude.lm` rather than a table held
//! beside it, so what it declares is read out of that source exactly as a module's declarations
//! are read out of its file. What stays here is how a module reaches one of them: by the spelling
//! it writes, because the prelude is a second file whose places are not the module's.

use std::sync::LazyLock;

use lumen_ast::{Item, Name};
use lumen_resolver::{Definition, ResolvedProgram, prelude};

use super::{Environment, Key};
use crate::error::TypeError;
use crate::scheme::{Quantified, Scheme};
use crate::table::Table;
use crate::types::{Type, TypeParameter};

impl Environment {
    /// Everything the prelude declares, ready for a module's own declarations on top of it.
    ///
    /// `docs/specs/library.md` says the compiler carries the prelude as Lumen source, so what it
    /// declares is read out of that source exactly as a module's is read out of its file. It is
    /// read once and copied, which is that spec's third property: the prelude's declarations are
    /// the same whichever module asks for them.
    pub(super) fn of_prelude() -> Self {
        let mut environment = DECLARED.clone();
        environment.keyed = Keyed::WhereWritten;
        environment
    }

    /// The prelude read out of the source the compiler carries, filed under the names it declares.
    ///
    /// A module reaches a prelude name by its spelling rather than by where it is written, which
    /// is what `Origin::Prelude` says, so the prelude is declared under those spellings.
    ///
    /// # Panics
    ///
    /// Panics where the carried prelude does not declare what it writes. That is the compiler's
    /// own failure and no program's, which `docs/specs/library.md` states, and the compiler's own
    /// tests are where it is caught.
    fn carried() -> Self {
        let resolved = lumen_resolver::prelude_resolved();
        let mut environment = Self {
            keyed: Keyed::ByName,
            ..Self::default()
        };
        for (name, takes) in prelude::held_types() {
            environment.arities.insert(Key::prelude(name), takes);
        }
        environment.bind(Key::prelude("todo"), todo());
        for (of, for_type) in prelude::still_supplied() {
            environment
                .instances
                .insert((of.to_owned(), for_type.to_owned()));
        }
        environment.note_arities(resolved);
        environment
            .declare_traits(resolved)
            .and_then(|()| environment.declare_reachable(resolved))
            .expect("the prelude the compiler carries declares what it writes");
        environment
    }

    /// The prelude's declarations a module reaches, which is every one it can write the name of.
    ///
    /// An instance's methods are not among them: a module reaches an instance by asking for the
    /// trait at a type rather than by naming the body that answers, and the body is checked where
    /// the prelude itself is compiled. Filing it here would file it under the trait method's own
    /// name, which is the name it answers for.
    ///
    /// # Errors
    ///
    /// Returns the first declaration that does not hold together.
    fn declare_reachable(&mut self, resolved: &ResolvedProgram) -> Result<(), TypeError> {
        for item in &resolved.program().items {
            match item {
                Item::Type(declaration) => self.declare_type(resolved, declaration)?,
                Item::Function(function) => {
                    self.declare_function(resolved, function, &mut Table::default())?;
                }
                Item::Instance(declaration) => self.note_instance(resolved, declaration)?,
                Item::Derive(declaration) => self.declare_derive(resolved, declaration)?,
                Item::Import(_) | Item::Trait(_) => {}
            }
        }
        Ok(())
    }

    /// The key the declaration `name` is the name of, in the source being read.
    pub(super) fn declared_at(&self, name: &Name) -> Key {
        match self.keyed {
            Keyed::WhereWritten => Key::at(name),
            Keyed::ByName => Key::prelude(&name.text),
        }
    }

    /// The key a use of `name` reaches, which `definition` says where to find.
    pub(super) fn key_of(&self, definition: Definition, name: &Name) -> Key {
        match self.keyed {
            Keyed::WhereWritten => Key::of(definition, name),
            Keyed::ByName => Key::prelude(&name.text),
        }
    }
}

/// The type one method of one prelude trait has, at the type `at`.
///
/// The prelude writes every one of these signatures itself, so this reads the declaration rather
/// than restating it: `docs/specs/traits.md` and `docs/specs/operators.md` are what they say, and
/// `library/prelude.lm` is where they are written down.
///
/// # Panics
///
/// Panics where the prelude declares no such trait, or no such method of it.
pub(crate) fn signature_of(of: &str, method: &str, at: &Type) -> Type {
    DECLARED
        .method_at(of, method, at)
        .expect("the prelude declares the trait and the method asked of it")
}

/// `todo(reason)`: a hole, which is whatever type the place it is written in expects.
///
/// `docs/specs/holes.md` states what it is for. It gives back a type nothing constrains, so a
/// hole unifies with whatever belongs where it is written.
fn todo() -> Scheme {
    let value = TypeParameter::prelude("T");
    Scheme::over(
        vec![Quantified::Parameter(value.clone())],
        Type::function(vec![Type::string()], Type::Parameter(value)),
    )
}

/// How a declaration of the source being read is filed.
///
/// A module's declarations are known by where they are written, which is unique within the one
/// file being compiled. The prelude is a second file, whose places would collide with that one,
/// and a module reaches a prelude name by spelling in any case, so it is filed by name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Keyed {
    #[default]
    WhereWritten,
    ByName,
}

/// The prelude's own declarations, read out of the carried source the first time one is asked for.
static DECLARED: LazyLock<Environment> = LazyLock::new(Environment::carried);
