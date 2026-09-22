//! What a call of `list.push` or of `list.at` runs, written out where the call stands.
//!
//! `docs/specs/library.md` says the two are the compiler's rather than the `list` module's, and
//! `docs/specs/codegen.md` says what each of them becomes. Neither is a method of a class, so
//! neither is ever called: a use is the instructions it always was, the way an operator over
//! `Int` is.

use lumen_ast::Expr;

use crate::code::{Comparison, Instruction, Label, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::body::{Builder, LIST, reaching};
use crate::lower::modules::Through;
use crate::lower::shape::{CONSTRUCTOR, NONE, SOME, object};

/// The one module either of them is reached through, which the library carries.
const MODULE: &str = "list";

/// The name of the one that grows a list.
const PUSH: &str = "push";

/// The name of the one that reads a list at an index.
const AT: &str = "at";

/// The growable list a push gathers into, which is the one a JVM already has.
const GATHERING: &str = "java/util/ArrayList";

/// Everything a growable list is built out of, which is what its constructor takes.
const COLLECTION: &str = "java/util/Collection";

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
        let [values, second] = arguments else {
            return None;
        };
        match reached.name.text.as_str() {
            PUSH => Some(self.pushed(values, second)),
            AT => Some(self.read_at(values, second)),
            _ => None,
        }
    }

    /// `list.push(values, value)`: the list gathered again, with the value after its last element.
    ///
    /// The gathering is a growable list, the value goes on the end of it, and the array it is
    /// read back as is the array a written list is built from. What comes back is a list built
    /// the one way a list is built, so the list the push was handed is untouched.
    fn pushed(&mut self, values: &Expr, value: &Expr) -> Descriptor {
        let holding = Descriptor::reference(LIST);
        self.emit(Instruction::New(ClassName::new(GATHERING)));
        self.emit(Instruction::Copy);
        self.handed(values, Some(holding.clone()));
        self.emit(Instruction::Construct(MethodRef {
            class: ClassName::new(GATHERING),
            name: CONSTRUCTOR.to_owned(),
            descriptor: MethodDescriptor::new(vec![Descriptor::reference(COLLECTION)], None),
        }));
        self.emit(Instruction::Copy);
        self.handed(value, Some(object()));
        self.emit(Instruction::InvokeVirtual(gathered(
            "add",
            vec![object()],
            Descriptor::Boolean,
        )));
        self.emit(Instruction::Drop(Descriptor::Boolean));
        self.emit(Instruction::InvokeVirtual(gathered(
            "toArray",
            Vec::new(),
            Descriptor::array(object()),
        )));
        self.emit(Instruction::CollectList);
        holding
    }

    /// `list.at(values, index)`: `Some` of the element there, and `None` past either end.
    ///
    /// Both stand in locals because the guard reads each of them twice. A call evaluates its
    /// arguments in the order they are written, and each is evaluated once.
    fn read_at(&mut self, values: &Expr, index: &Expr) -> Descriptor {
        let holding = Descriptor::reference(LIST);
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
    /// and below a size a JVM counts in a small whole number, so a small whole number holds it.
    fn within(&mut self, list: u16, at: u16, empty: Label) {
        self.loaded_index(at);
        self.emit(Instruction::Long(0));
        self.emit(Instruction::CompareLongs(Comparison::GreaterOrEqual));
        self.emit(Instruction::JumpIfFalse(empty));
        self.loaded_index(at);
        self.loaded_list(list);
        self.emit(Instruction::InvokeInterface(reaching(
            "size",
            Vec::new(),
            Descriptor::Integer,
        )));
        self.emit(Instruction::Widen);
        self.emit(Instruction::CompareLongs(Comparison::Less));
        self.emit(Instruction::JumpIfFalse(empty));
    }

    /// The element the list holds at the index, carried as the reference a `Some` holds.
    fn some_of(&mut self, list: u16, at: u16) -> Descriptor {
        let shape = self.lowering.shapes.built(SOME).clone();
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        self.loaded_list(list);
        self.loaded_index(at);
        self.emit(Instruction::Narrow);
        self.emit(Instruction::InvokeInterface(reaching(
            "get",
            vec![Descriptor::Integer],
            object(),
        )));
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
            of: Descriptor::reference(LIST),
        });
    }

    fn loaded_index(&mut self, at: u16) {
        self.emit(Instruction::Load {
            slot: at,
            of: Descriptor::Long,
        });
    }
}

/// A method of the growable list a push gathers into.
fn gathered(name: &str, parameters: Vec<Descriptor>, result: Descriptor) -> MethodRef {
    MethodRef {
        class: ClassName::new(GATHERING),
        name: name.to_owned(),
        descriptor: MethodDescriptor::new(parameters, Some(result)),
    }
}
