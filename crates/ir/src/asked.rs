//! The methods one module asks another for, which is what a generic reached through an import is.
//!
//! `docs/specs/codegen.md` writes a generic once per set of types it is used at, in the class of
//! the module that declares it. A use in another module settles the set where it is written, so
//! the set travels from the module writing the use to the module writing the method, and this is
//! what it travels as.

use lumen_types::Type;

/// Every method a build has asked a module for, in the order they were asked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Asked {
    of: Vec<Specialisation>,
}

impl Asked {
    /// Everything `module` has been asked for, which is what its own lowering writes on top of
    /// what its body reaches.
    pub(crate) fn of_module<'a>(
        &'a self,
        module: &'a str,
    ) -> impl Iterator<Item = &'a Specialisation> {
        self.of.iter().filter(move |one| one.module == module)
    }

    /// The same, with everything `other` asked for asked as well.
    #[must_use]
    pub fn and(mut self, other: &Self) -> Self {
        for one in &other.of {
            self.note(one.clone());
        }
        self
    }

    /// Asks for `one`, unless the same set of types has already been asked for.
    pub(crate) fn note(&mut self, one: Specialisation) {
        if !self.of.contains(&one) {
            self.of.push(one);
        }
    }
}

/// One use of another module's generic, as the module writing the use reads it.
///
/// The set it settled is what the other module writes the method for, and the constraints that
/// module declared are what say how the method is named; `docs/specs/codegen.md` states both.
pub(crate) struct Asking {
    pub(crate) settled: Vec<Type>,
    pub(crate) constrained: Vec<Vec<String>>,
}

/// One method a module owes another: a generic it declares, at one set of types.
///
/// The types are written as the module being asked writes them, which `docs/specs/codegen.md`
/// states, so the module writing the method reads them as it reads its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Specialisation {
    module: String,
    function: String,
    settled: Vec<Type>,
}

impl Specialisation {
    /// A use of `function` in `module`, which settled its type parameters on `settled`.
    pub(crate) fn of(module: &str, function: &str, settled: Vec<Type>) -> Self {
        Self {
            module: module.to_owned(),
            function: function.to_owned(),
            settled,
        }
    }

    /// The name the module being asked declares it under.
    pub(crate) fn function(&self) -> &str {
        &self.function
    }

    /// What the use settled each type parameter on, in the order they are declared.
    pub(crate) fn settled(&self) -> &[Type] {
        &self.settled
    }
}
