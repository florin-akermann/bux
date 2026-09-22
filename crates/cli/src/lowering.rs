//! Every module a program reaches, and the prelude, lowered to the classes a build writes.
//!
//! A generic is written by the module that declares it, at each set of types a use settled it
//! at, which `docs/specs/codegen.md` states, and a use in one module asks the module it reaches
//! into. An instance of a type is written by the module that declares the type, and the prelude
//! writes its instances over a list; a use asks for those in the same way.

use lumen_diagnostics::Diagnostic;
use lumen_examples::{Refusal as ExampleRefusal, stated_by};
use lumen_holes::{Hole, Whole};
use lumen_ir::{Asked, Lowered, lower, lower_prelude};
use lumen_resolver::library::PRELUDE;
use lumen_types::TypedProgram;

use crate::compiling::{Module, NotCompiled};

/// Every module of `modules` and the prelude, lowered, in the order their classes are written.
///
/// A module is lowered after everything that imports it: loading orders them dependencies first,
/// and a pass runs that order backwards, the prelude last, and turns the answer back around.
/// An import is not the only way one module asks another, because a body of the prelude or of a
/// generic instance can reach an instance of a module that was lowered before it. So a pass that
/// asks a module for more than that module was lowered knowing is run again with every ask found,
/// and the pass that asks nothing new is the one written.
pub(crate) fn lowered(modules: &[Module]) -> Result<Vec<Lowered>, NotCompiled> {
    let mut asked = Asked::default();
    loop {
        let pass = Pass::over(modules, asked)?;
        if pass.asks_nothing_new() {
            return Ok(pass.written());
        }
        asked = pass.asked;
    }
}

/// One pass over every module and the prelude, and everything they asked of each other.
struct Pass {
    lowered: Vec<Lowered>,
    /// Every ask the pass started with, and every ask it found.
    asked: Asked,
    /// Each module the pass lowered, with every ask it knew of when it was lowered.
    knew: Vec<(String, Asked)>,
}

impl Pass {
    /// Lowers each of `modules` after everything that imports it, then the prelude.
    fn over(modules: &[Module], asked: Asked) -> Result<Self, NotCompiled> {
        let mut pass = Self {
            lowered: Vec::new(),
            asked,
            knew: Vec::new(),
        };
        for module in modules.iter().rev() {
            let one = lowered_from(module, &pass.asked)?;
            pass.add(module.name(), one);
        }
        let prelude = lower_prelude(&pass.asked);
        pass.add(PRELUDE, prelude);
        Ok(pass)
    }

    /// Adds what `module` became, and what it asked the modules after it for.
    fn add(&mut self, module: &str, one: Lowered) {
        self.knew.push((module.to_owned(), self.asked.clone()));
        self.asked = std::mem::take(&mut self.asked).and(&one.asks);
        self.lowered.push(one);
    }

    /// Whether every module was lowered knowing every ask of it the whole pass found.
    fn asks_nothing_new(&self) -> bool {
        self.knew
            .iter()
            .all(|(module, knew)| !self.asked.asks_more_of(module, knew))
    }

    /// The classes of the pass, in the order they are written: the prelude, then dependencies.
    fn written(mut self) -> Vec<Lowered> {
        self.lowered.reverse();
        self.lowered
    }
}

/// The module `module` becomes, or every refusal that stops it becoming one.
fn lowered_from(module: &Module, asked: &Asked) -> Result<Lowered, NotCompiled> {
    compiled(module.typed(), module.source(), module.name(), asked)
        .map_err(|refusals| NotCompiled::refused(refusals, module.path(), module.source()))
}

/// The module `typed` becomes, or every refusal that stops it becoming one.
///
/// Two things a build asks of a module that a check does not are asked here: every body is
/// written, which `docs/specs/holes.md` states, and every function says what it does, which
/// `docs/specs/doc-examples.md` states.
fn compiled(
    typed: &TypedProgram,
    source: &str,
    module: &str,
    asked: &Asked,
) -> Result<Lowered, Vec<Diagnostic>> {
    let whole = Whole::of_module(typed)
        .map_err(|holes| holes.iter().map(Hole::diagnostic).collect::<Vec<_>>())?;
    stated_by(source, typed.resolved().program()).map_err(|refused| {
        refused
            .iter()
            .map(ExampleRefusal::diagnostic)
            .collect::<Vec<_>>()
    })?;
    Ok(lower(&whole, module, asked))
}
