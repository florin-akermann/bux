//! What a call of `list.push` or of `list.at` runs, written out where the call stands.
//!
//! `docs/specs/library.md` says the two are the compiler's rather than the `list` module's, and
//! `docs/specs/codegen.md` says what each of them becomes. Neither is a method of the `list`
//! class: `at` is the instructions it always was, the way an operator over `Int` is, and `push`
//! is a call of the class that carries a list, because a push branches and copies.

use lumen_ast::Expr;

use crate::code::{Comparison, Instruction, Label};
use crate::descriptor::Descriptor;
use crate::lower::body::Builder;
use crate::lower::carrier;
use crate::lower::modules::Through;
use crate::lower::shape::{NONE, SOME, object};

/// The one module either of them is reached through, which the library carries.
const MODULE: &str = "list";

/// The name of the one that grows a list.
const PUSH: &str = "push";

/// The name of the one that reads a list at an index.
const AT: &str = "at";

impl Builder<'_> {
    /// What the call runs, where it is a call of one of the two the compiler holds.
    ///
    /// Every other call through a module is a call of a method of that module's class, which
    /// `docs/specs/codegen.md` states, so nothing else answers here.
    pub(crate) fn held_by_the_compiler(
        &mut self,
        reached: &Through<'_>,
        arguments: &[&Expr],
    ) -> Option<Descriptor> {
        if reached.module.text != MODULE {
            return None;
        }
        self.held_named(&reached.name.text, arguments)
    }

    /// What the call of `name` runs, where `name` is one of the two the compiler holds.
    ///
    /// The prelude's own source writes `at` without `list` in front, which
    /// `docs/specs/library.md` states, so its instances over a list reach it by the name alone.
    pub(crate) fn held_named(&mut self, name: &str, arguments: &[&Expr]) -> Option<Descriptor> {
        let [values, second] = arguments else {
            return None;
        };
        match name {
            PUSH => Some(self.pushed(values, second)),
            AT => Some(self.read_at(values, second)),
            _ => None,
        }
    }

    /// `list.push(values, value)`: the list with the value after its last element.
    ///
    /// It is a call of `lumen.List.push`, which claims the next slot of the buffer where it is
    /// free and copies first where it is not, so the list the push was handed is untouched.
    fn pushed(&mut self, values: &Expr, value: &Expr) -> Descriptor {
        self.handed(values, Some(carrier::list()));
        self.handed(value, Some(object()));
        self.emit(Instruction::InvokeStatic(carrier::pushed()));
        carrier::list()
    }

    /// `list.at(values, index)`: `Some` of the element there, and `None` past either end.
    ///
    /// Both stand in locals because the guard reads each of them twice. A call evaluates its
    /// arguments in the order they are written, and each is evaluated once.
    fn read_at(&mut self, values: &Expr, index: &Expr) -> Descriptor {
        let holding = carrier::list();
        self.handed(values, Some(holding.clone()));
        let list = self.temporary(&holding);
        self.emit(Instruction::Store {
            slot: list,
            of: holding,
        });
        self.handed(index, Some(Descriptor::Long));
        let at = self.temporary(&Descriptor::Long);
        self.emit(Instruction::Store {
            slot: at,
            of: Descriptor::Long,
        });
        let empty = self.label();
        let end = self.label();
        self.within(list, at, empty);
        let held = self.some_of(list, at);
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(empty));
        self.none();
        self.emit(Instruction::Label(end));
        held
    }

    /// Jumps to `empty` unless the index is one the list holds, which is the whole of the guard.
    ///
    /// It is what makes the narrowing below total: an index that gets past here is at least zero
    /// and below a length a JVM counts in a small whole number, so a small whole number holds it.
    fn within(&mut self, list: u16, at: u16, empty: Label) {
        self.loaded_index(at);
        self.emit(Instruction::Long(0));
        self.emit(Instruction::CompareLongs(Comparison::GreaterOrEqual));
        self.emit(Instruction::JumpIfFalse(empty));
        self.loaded_index(at);
        self.loaded_list(list);
        self.emit(Instruction::GetField(carrier::length()));
        self.emit(Instruction::Widen);
        self.emit(Instruction::CompareLongs(Comparison::Less));
        self.emit(Instruction::JumpIfFalse(empty));
    }

    /// The element the buffer holds at the index, carried as the reference a `Some` holds.
    fn some_of(&mut self, list: u16, at: u16) -> Descriptor {
        let shape = self.lowering.shapes.built(SOME).clone();
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        self.loaded_list(list);
        self.emit(Instruction::GetField(carrier::slots()));
        self.loaded_index(at);
        self.emit(Instruction::Narrow);
        self.emit(Instruction::LoadFromArray);
        self.constructed(&shape)
    }

    /// The answer where the list holds no element at the index, which carries nothing.
    fn none(&mut self) {
        let shape = self.lowering.shapes.built(NONE).clone();
        self.builds(&shape, &[]);
    }

    fn loaded_list(&mut self, list: u16) {
        self.emit(Instruction::Load {
            slot: list,
            of: carrier::list(),
        });
    }

    fn loaded_index(&mut self, at: u16) {
        self.emit(Instruction::Load {
            slot: at,
            of: Descriptor::Long,
        });
    }
}
