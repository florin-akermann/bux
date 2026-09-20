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
use crate::infer;
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
    /// trait at a type rather than by naming the body that answers, and filing one here would
    /// file it under the trait method's own name, which is the name it answers for. What each of
    /// those bodies amounts to is the JVM instruction for it, which `docs/specs/library.md`
    /// states along with why version 0.1 does not read them.
    ///
    /// # Errors
    ///
    /// Returns the first declaration that does not hold together.
    ///
    /// # Panics
    ///
    /// Panics where the prelude leaves a type to be inferred. A module is inferred against a
    /// table of its own, and a variable made here would be a variable that table hands out
    /// again, so the prelude writes every type it declares rather than leaving one open.
    fn declare_reachable(&mut self, resolved: &ResolvedProgram) -> Result<(), TypeError> {
        let mut table = Table::default();
        for item in &resolved.program().items {
            match item {
                Item::Type(declaration) => self.declare_type(resolved, declaration)?,
                Item::Function(function) => {
                    self.declare_function(resolved, function, &mut table)?;
                }
                Item::Instance(declaration) => self.note_instance(resolved, declaration)?,
                Item::Derive(declaration) => self.declare_derive(resolved, declaration)?,
                Item::Import(_) | Item::Trait(_) => {}
            }
        }
        assert_eq!(
            table.made(),
            0,
            "the prelude writes out the type of everything it declares"
        );
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

/// Walks the prelude's own bodies, once however many modules a build checks.
///
/// It is done before the first module is, so a library that does not hold together is the
/// compiler's own failure before it is anything else, which `docs/specs/library.md` states.
pub(crate) fn prelude_checked() {
    LazyLock::force(&CHECKED);
}

/// Whether the prelude itself has the instance of `of` for the type called `at`.
///
/// A module's own instances are its own: `docs/specs/modules.md` keeps a trait and its instances
/// where they are declared, so the prelude's are the only ones two modules both reach.
pub(crate) fn of_the_prelude(of: &str, at: &str) -> bool {
    DECLARED.has_instance(of, at)
}

/// The type one method of one prelude trait has, at the type `at`.
///
/// The prelude writes every one of these signatures itself, so this reads the declaration rather
/// than restating it: `docs/specs/traits.md` and `docs/specs/operators.md` are what they say, and
/// `library/prelude.lm` is where they are written down.
///
/// It is asked only once the prelude has been read: a derive is what asks, and reading the prelude
/// declares a derive without reading one. Asking while the prelude is still being read would ask
/// for what is being worked out, which is why `declare_reachable` records a derive and checks
/// none.
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

/// The prelude's own bodies, walked once against what reading it declared.
///
/// `docs/specs/library.md` says the library is compiled with every check a program is compiled
/// with, so what its source says an instruction does is held to the declarations above it rather
/// than read by a person alone. It is walked where the prelude is read, which is the first time
/// any module asks for it, and once for however many modules a build has.
///
/// # Panics
///
/// Panics where a body of the prelude does not have the type its declaration gives it. That is
/// the compiler's own failure and no program's, which `docs/specs/library.md` states, and the
/// compiler's own tests are where it is caught.
static CHECKED: LazyLock<()> = LazyLock::new(|| {
    infer::of_the_prelude().expect("the prelude the compiler carries is a module that compiles");
});
