//! Type inference: every expression of a module given the type it has.
//!
//! The phase consumes the resolved tree and yields a [`TypedProgram`], where each expression
//! carries the type inference gave it. Nothing is mutated: the tree comes through unchanged and
//! the answers sit beside it. `docs/specs/types.md` is the specification, and `docs/design.md`
//! sections 3, 6, and 7 are the rules it enforces.

mod boundary;
mod bounds;
mod derive;
mod environment;
mod error;
mod held;
mod holds;
mod infer;
mod scheme;
mod surface;
mod table;
mod types;
mod unify;

use std::collections::HashMap;
use std::sync::LazyLock;

use lumen_ast::Span;
use lumen_resolver::ResolvedProgram;

pub use crate::error::TypeError;
pub use crate::surface::{BuiltBy, GenericUse, Imported, OfferedConstructor};
pub use crate::surface::{OfferedType, Surface};
pub use crate::types::{Type, TypeParameter, TypeVar};

/// The type the method `method` of the prelude's trait `trait_name` has, at the type `for_type`.
///
/// A generic written for a type another module declares calls that module's instance, and this
/// is what that call takes and gives back: `docs/specs/codegen.md` states it. The prelude writes
/// every one of these signatures, so this reads the declaration rather than restating it.
///
/// # Panics
///
/// Panics where the prelude declares no such trait, or no such method of it.
#[must_use]
pub fn instance_signature(trait_name: &str, method: &str, for_type: &Type) -> Type {
    environment::signature_of(trait_name, method, for_type)
}

/// Gives every expression of `resolved` the type it has, reaching `imported` through an import.
///
/// # Errors
///
/// Returns the first declaration that holds a value of itself, or, where none does, the first
/// expression whose type inference cannot give it.
pub fn check(resolved: ResolvedProgram, imported: &Imported) -> Result<TypedProgram, TypeError> {
    LazyLock::force(&PRELUDE);
    holds::nothing_holds_itself(&resolved)?;
    let reached = imported.every_type();
    let inferred = infer::infer(&resolved, imported, &reached)?;
    Ok(TypedProgram::of(resolved, inferred, reached))
}

/// The prelude, with every expression of its bodies given the type it has.
///
/// The prelude's instances over a list are lowered from their bodies, so lowering reads the
/// prelude as it reads any module; `docs/specs/codegen.md` states the class they are written into.
#[must_use]
pub fn prelude() -> &'static TypedProgram {
    &PRELUDE
}

/// The prelude's own bodies, walked once against what reading it declared.
///
/// `docs/specs/library.md` says the library is compiled with every check a program is compiled
/// with, so what its source says an instruction does is held to the declarations above it rather
/// than read by a person alone. It is walked the first time any module is checked, and once for
/// however many modules a build has, so a library that does not hold together is the compiler's
/// own failure before it is anything else.
///
/// # Panics
///
/// Panics where a body of the prelude does not have the type its declaration gives it. That is
/// the compiler's own failure and no program's, which `docs/specs/library.md` states, and the
/// compiler's own tests are where it is caught.
static PRELUDE: LazyLock<TypedProgram> = LazyLock::new(|| {
    let resolved = lumen_resolver::prelude_resolved().clone();
    let inferred = infer::of_the_prelude()
        .expect("the prelude the compiler carries is a module that compiles");
    TypedProgram::of(resolved, inferred, Vec::new())
});

/// A program whose every expression, and every name that declares one, has the type it has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedProgram {
    resolved: ResolvedProgram,
    types: HashMap<Span, Type>,
    surface: Surface,
    methods: HashMap<Span, Type>,
    generics_reached: HashMap<Span, GenericUse>,
    reached: Vec<OfferedType>,
}

impl TypedProgram {
    fn of(resolved: ResolvedProgram, inferred: infer::Inferred, reached: Vec<OfferedType>) -> Self {
        Self {
            resolved,
            types: inferred.types,
            surface: inferred.surface,
            methods: inferred.methods,
            generics_reached: inferred.generics_reached,
            reached,
        }
    }

    /// The program this was inferred from, with every name still pointed at its definition.
    #[must_use]
    pub const fn resolved(&self) -> &ResolvedProgram {
        &self.resolved
    }

    /// What this module offers the modules that import it.
    #[must_use]
    pub const fn surface(&self) -> &Surface {
        &self.surface
    }

    /// Every type the modules this one imports offer, under the name this one writes it by.
    ///
    /// A type of another module is that module's own, so what it is built with and what each of
    /// those carries is read from here rather than from anything this module declares.
    #[must_use]
    pub fn reached(&self) -> &[OfferedType] {
        &self.reached
    }

    /// The type of what is written at `at`.
    ///
    /// Every expression has one, and so does every name that declares something: a function, a
    /// parameter, and each name a binding or a pattern introduces.
    #[must_use]
    pub fn type_of(&self, written: Span) -> Option<&Type> {
        self.types.get(&written)
    }

    /// The type a use of a trait method at `written` reached its instance at.
    ///
    /// Every use of a trait method has one, and nothing else does: that is what tells lowering
    /// which instance the call is a call of, which `docs/specs/traits.md` states is settled
    /// while the program is compiled.
    #[must_use]
    pub fn instance_at(&self, written: Span) -> Option<&Type> {
        self.methods.get(&written)
    }

    /// What the use at `written` reaches, where what it reaches is another module's generic.
    ///
    /// Only a use reached through an import has one: `docs/specs/codegen.md` writes a generic
    /// once per set of types it is used at, by the module that declares it, so this says which
    /// set that module is asked for and what the method written for it takes and gives back.
    #[must_use]
    pub fn generic_reached(&self, written: Span) -> Option<&GenericUse> {
        self.generics_reached.get(&written)
    }
}
