//! The instances that travel with a type, out of the module that declares it and into another.
//!
//! `docs/design.md` section 8 puts an instance beside its trait or beside its type, and one trait
//! and one type have one instance in a program. So a module that reaches a type of another module
//! reaches the instances that module gives it, which `docs/specs/modules.md` states.

use lumen_ast::{Item, Program};
use lumen_resolver::{ResolvedProgram, prelude};

use super::Environment;
use crate::bounds::Bounds;
use crate::surface::OfferedInstance;

impl Environment {
    /// One instance another module gives a type it declares, which travels with that type.
    pub(super) fn given_by_another_module(&mut self, given: &OfferedInstance) {
        let key = (given.of.clone(), given.for_type.clone());
        self.instances.insert(key, given.asks.clone());
        if let Some(bounds) = given.holds {
            self.holds.insert(given.for_type.clone(), bounds);
        }
    }

    /// Every instance of the prelude's traits this module gives a type it declares.
    ///
    /// An instance of a trait the module declares stays here, because no other module can name
    /// that trait. So does an instance whose constraint names such a trait, because no other
    /// module can answer that constraint. They come back in the order their names sort in, so what is offered does not
    /// depend on the order a table happened to hold them in.
    pub(crate) fn given(&self, resolved: &ResolvedProgram) -> Vec<OfferedInstance> {
        let program = resolved.program();
        let mut given: Vec<OfferedInstance> = self
            .instances
            .iter()
            .filter(|((of, for_type), asks)| {
                declares_type(program, for_type)
                    && !declares_trait(program, of)
                    && !asks
                        .iter()
                        .flatten()
                        .any(|asked| declares_trait(program, asked))
            })
            .map(|((of, for_type), asks)| OfferedInstance {
                of: of.clone(),
                for_type: for_type.clone(),
                asks: asks.clone(),
                holds: self.holds_as(of, for_type),
            })
            .collect();
        given.sort_by(|one, other| (&one.for_type, &one.of).cmp(&(&other.for_type, &other.of)));
        given
    }

    /// What the type called `for_type` holds, where `of` is the trait that says so.
    fn holds_as(&self, of: &str, for_type: &str) -> Option<Bounds> {
        if of == prelude::INTEGER_LITERAL {
            return self.holds(for_type);
        }
        None
    }
}

/// Whether `program` declares a type called `named`.
fn declares_type(program: &Program, named: &str) -> bool {
    program
        .items
        .iter()
        .any(|item| matches!(item, Item::Type(declaration) if declaration.name.text == named))
}

/// Whether `program` declares a trait called `named`.
fn declares_trait(program: &Program, named: &str) -> bool {
    program
        .items
        .iter()
        .any(|item| matches!(item, Item::Trait(declaration) if declaration.name.text == named))
}
