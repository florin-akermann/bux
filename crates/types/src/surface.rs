//! What a module offers the modules that import it, and what they offer back.
//!
//! `docs/specs/modules.md` states what that is: every function a module declares and every type
//! it declares, each reached through the module's name. A type is offered whole, with what
//! builds it and what each of those carries, because an importing module writes values of it and
//! matches them. A trait and its instances are not offered: they stay where they are declared.

use std::collections::{HashMap, HashSet};

use crate::scheme::{Quantified, Scheme};
use crate::types::Type;

/// Every function and every type a module declares, as a module importing it writes them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Surface {
    functions: HashMap<String, Offered>,
    types: Vec<OfferedType>,
}

/// What one function of a module amounts to for a module that imports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Offered {
    /// A function written once, whose type is fresh at each use.
    Plain(Scheme),
    /// A function written once per set of types it is used at, which `docs/specs/codegen.md`
    /// states, so a use of one says which set it settled and the declaring module writes it.
    Generic(Scheme),
}

impl Offered {
    fn reached_as(self, module: &str, own: &HashSet<String>) -> Self {
        let renamed =
            |scheme: Scheme| scheme.renamed(&|written| reached_through(written, module, own));
        match self {
            Self::Generic(scheme) => Self::Generic(renamed(scheme)),
            Self::Plain(scheme) => Self::Plain(renamed(scheme)),
        }
    }

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
            Self::Generic(scheme)
        }
    }
}

/// What one use of a generic another module declares reaches.
///
/// `docs/specs/codegen.md` writes a generic once per set of types it is used at, by the module
/// that declares it, so a use of one through an import is a call of the method written for the
/// set that use settled. These are the two things such a call needs: which set to ask for, and
/// the type the method written for it has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericUse {
    settled: Vec<Type>,
    written_as: Type,
}

impl GenericUse {
    pub(crate) const fn reaching(settled: Vec<Type>, written_as: Type) -> Self {
        Self {
            settled,
            written_as,
        }
    }

    /// The same, with `solve` applied to every type it holds.
    pub(crate) fn solved(&self, solve: &impl Fn(&Type) -> Type) -> Self {
        Self {
            settled: self.settled.iter().map(solve).collect(),
            written_as: solve(&self.written_as),
        }
    }

    /// What the use settled each type parameter on, in the order the declaration writes them.
    #[must_use]
    pub fn settled(&self) -> &[Type] {
        &self.settled
    }

    /// The type the method written for this use has, which is what a call of it is written with.
    ///
    /// It is not the type the use has: a stand-in inference made rather than the source wrote is
    /// carried by what every value fits, so a use of one at an `Int` still reaches a method that
    /// takes and gives back that.
    #[must_use]
    pub const fn written_as(&self) -> &Type {
        &self.written_as
    }
}

/// One type a module offers: the name it is written by, and what builds a value of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferedType {
    name: String,
    over: Vec<Quantified>,
    declared: Type,
    built_by: BuiltBy,
}

/// What a type a module offers is built with: one constructor, or one for each variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuiltBy {
    /// A record, built with the one constructor its own name is, whose fields a value has.
    Record(OfferedConstructor),
    /// An algebraic data type, in the order it declares its variants.
    Variants(Vec<OfferedConstructor>),
    /// A Java class, named as its `extern type` writes it, built by an `extern new` alone.
    Foreign(String),
}

/// One constructor a type is built with, and what a value it builds carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferedConstructor {
    name: String,
    /// What each value it carries is named, which is empty where it carries them in order.
    labels: Vec<String>,
    carries: Vec<Type>,
}

impl OfferedType {
    /// One type as it is declared, which is how the module declaring it writes it.
    pub(crate) fn declared(
        name: String,
        over: Vec<Quantified>,
        declared: Type,
        built_by: BuiltBy,
    ) -> Self {
        Self {
            name,
            over,
            declared,
            built_by,
        }
    }

    /// The name this is written by, which is the module's name and the declaration's.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What builds a value of it, which is what a use of it is read against.
    #[must_use]
    pub const fn built_by(&self) -> &BuiltBy {
        &self.built_by
    }

    /// How many type arguments it is written with, which every use of it is held to.
    #[must_use]
    pub const fn arity(&self) -> usize {
        self.over.len()
    }

    /// Every constructor it has, in the order it declares them, which is a tag's order.
    pub(crate) fn constructors(&self) -> &[OfferedConstructor] {
        match &self.built_by {
            BuiltBy::Record(one) => std::slice::from_ref(one),
            BuiltBy::Variants(variants) => variants,
            // Nothing in Lumen builds a Java class; an `extern new` is what reaches one.
            BuiltBy::Foreign(_) => &[],
        }
    }

    fn reached_as(self, module: &str, own: &HashSet<String>) -> Self {
        Self {
            name: reached(&self.name, module),
            over: self.over,
            declared: reached_through(&self.declared, module, own),
            built_by: self.built_by.reached_as(module, own),
        }
    }

    /// The scheme one of its constructors has, which is a function of what that one carries.
    pub(crate) fn scheme_of(&self, built: &OfferedConstructor) -> Scheme {
        let body = if built.carries.is_empty() {
            self.declared.clone()
        } else {
            Type::function(built.carries.clone(), self.declared.clone())
        };
        Scheme::over(self.over.clone(), body)
    }
}

impl OfferedConstructor {
    /// One constructor as it is declared, which is how the module declaring it builds one.
    pub(crate) fn declared(name: String, labels: Vec<String>, carries: Vec<Type>) -> Self {
        Self {
            name,
            labels,
            carries,
        }
    }

    fn reached_as(self, module: &str, own: &HashSet<String>) -> Self {
        Self {
            name: reached(&self.name, module),
            labels: self.labels,
            carries: self
                .carries
                .iter()
                .map(|carried| reached_through(carried, module, own))
                .collect(),
        }
    }

    /// The name it is written by, which is the module's name and the constructor's.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What each value it carries is named, which is empty where it carries them in order.
    #[must_use]
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// The type of each value it carries, in the order it carries them.
    #[must_use]
    pub fn carries(&self) -> &[Type] {
        &self.carries
    }
}

impl Surface {
    /// What a module offers: the scheme of each of its functions, and each type it declares.
    pub(crate) fn of(functions: HashMap<String, Scheme>, types: Vec<OfferedType>) -> Self {
        let offered = functions
            .into_iter()
            .map(|(name, scheme)| (name, Offered::of(scheme)))
            .collect();
        Self {
            functions: offered,
            types,
        }
    }

    /// The same surface, with every name of it written as a module importing it writes it.
    ///
    /// A type is a bare name where it is declared and a name reached through the module
    /// everywhere else, so this is where the one becomes the other.
    fn reached_as(self, module: &str) -> Self {
        let own: HashSet<String> = self
            .types
            .iter()
            .map(|offered| offered.name.clone())
            .collect();
        let functions = self
            .functions
            .into_iter()
            .map(|(name, offered)| (name, offered.reached_as(module, &own)))
            .collect();
        let types = self
            .types
            .into_iter()
            .map(|offered| offered.reached_as(module, &own))
            .collect();
        Self { functions, types }
    }

    /// What the module offers under `name`, when it declares a function of that name.
    pub(crate) fn function(&self, name: &str) -> Option<&Offered> {
        self.functions.get(name)
    }
}

impl BuiltBy {
    fn reached_as(self, module: &str, own: &HashSet<String>) -> Self {
        match self {
            Self::Record(one) => Self::Record(one.reached_as(module, own)),
            // A Java class is named the same wherever it is reached from; it is the JVM's.
            Self::Foreign(class) => Self::Foreign(class),
            Self::Variants(variants) => Self::Variants(
                variants
                    .into_iter()
                    .map(|one| one.reached_as(module, own))
                    .collect(),
            ),
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
        self.surfaces
            .insert(module.to_owned(), surface.reached_as(module));
        self
    }

    /// What `module` offers, when it is one of the modules loaded.
    pub(crate) fn surface(&self, module: &str) -> Option<&Surface> {
        self.surfaces.get(module)
    }

    /// Every type every module loaded so far offers, under the names they are written by.
    ///
    /// A module this one does not import is no less reachable for it: a function of a module it
    /// does import gives back what it gives back, and that is the type its own module declared.
    /// Such a type is one this module holds and reads and never one it writes, because writing a
    /// name reached through a module needs that module in scope, which only an import puts it in.
    /// They come back in the order their names sort in, so what is offered does not depend on
    /// the order the modules happened to be loaded in.
    pub(crate) fn every_type(&self) -> Vec<OfferedType> {
        let mut offered: Vec<OfferedType> = self
            .surfaces
            .values()
            .flat_map(|surface| surface.types.iter().cloned())
            .collect();
        offered.sort_by(|one, other| one.name.cmp(&other.name));
        offered
    }
}

/// `written` with every type of `own` reached through `module`, and every other left alone.
fn reached_through(written: &Type, module: &str, own: &HashSet<String>) -> Type {
    match written {
        Type::Named { name, arguments } => Type::Named {
            name: if own.contains(name) {
                reached(name, module)
            } else {
                name.clone()
            },
            arguments: arguments
                .iter()
                .map(|argument| reached_through(argument, module, own))
                .collect(),
        },
        Type::Function { parameters, result } => Type::function(
            parameters
                .iter()
                .map(|parameter| reached_through(parameter, module, own))
                .collect(),
            reached_through(result, module, own),
        ),
        Type::Var(_) | Type::Parameter(_) | Type::Module(_) | Type::Unit => written.clone(),
    }
}

/// `User` as `demo.User`: one name, written as a module importing it writes it.
fn reached(name: &str, module: &str) -> String {
    format!("{module}.{name}")
}
