//! One function body, as the instructions and locals a method runs.

use std::collections::{HashMap, HashSet};

use lumen_ast::{AssignOperator, Block, Expr, ExprKind, ForHeader, ForLoop, Function, Name, Span};
use lumen_ast::{Statement, StatementKind};
use lumen_resolver::{Definition, Namespace, Origin, ResolvedProgram};
use lumen_types::Type;

use crate::code::{Body, Instruction, Label, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::generic::Instantiation;
use crate::lower::shape::{CONSTRUCTOR, object, object_class};
use crate::lower::{Lowering, Reaching, Signature, escape};

/// The list a `for … in` walks and a written list builds, which `docs/specs/codegen.md` names.
pub(crate) const LIST: &str = "java/util/List";

/// Where a binding lives, which is one local of the method the function became.
#[derive(Clone, Debug)]
pub(crate) struct Slot {
    pub(crate) at: u16,
    pub(crate) of: Descriptor,
}

/// One field of a binding kept in locals: the name that declares it, and the field's own name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Held {
    pub(crate) binding: Span,
    pub(crate) field: String,
}

/// The two places a loop is left from, which is what `break` and `continue` jump to.
struct Repeat {
    again: Label,
    done: Label,
}

/// What one turn of a loop does before its body, which is what its header says.
enum Turn<'a> {
    /// `for { … }`, which only a `break` leaves.
    Forever,
    /// `for condition { … }`.
    While(&'a Expr),
    /// `for name in list { … }`, which counts through the list it was given.
    Walk {
        list: u16,
        index: u16,
        item: Option<Slot>,
    },
}

/// One body part-way through being lowered.
pub(crate) struct Builder<'a> {
    pub(crate) lowering: &'a Lowering<'a>,
    instructions: Vec<Instruction>,
    /// Where each binding lives, by the span of the name that declares it.
    slots: HashMap<Span, Slot>,
    next_slot: u16,
    /// The slot after the parameters, which is where the body's own locals begin.
    first_local: u16,
    next_label: u32,
    /// The loops the instruction being lowered is inside, the innermost one last.
    loops: Vec<Repeat>,
    /// The bindings kept in locals rather than built, by the span of the name declaring each.
    splitting: HashSet<Span>,
    /// Where each field of a split binding lives; a field carried by nothing lives nowhere.
    pub(crate) fields: HashMap<Held, Slot>,
    /// What the function gives back, which `return` and `?` both answer to.
    result: Option<Descriptor>,
    /// What the use this body is being written for settled each of its type parameters at.
    at: Instantiation,
}

impl<'a> Builder<'a> {
    /// A body about to be lowered, with a local for each parameter the function takes.
    pub(crate) fn entering(
        lowering: &'a Lowering<'a>,
        function: &Function,
        signature: &Signature,
        at: Instantiation,
    ) -> Self {
        let mut builder = Self {
            lowering,
            instructions: Vec::new(),
            slots: HashMap::new(),
            next_slot: 0,
            first_local: 0,
            next_label: 0,
            loops: Vec::new(),
            splitting: escape::split(lowering.typed.resolved(), &lowering.shapes, function),
            fields: HashMap::new(),
            result: signature.result.clone(),
            at,
        };
        for parameter in &function.parameters {
            builder.declare(&parameter.name);
        }
        builder.first_local = builder.next_slot;
        builder
    }

    /// Lowers `block` as the whole of a function, which always leaves the method.
    pub(crate) fn body(&mut self, block: &Block) {
        let left = self.block(block);
        let result = self.result.clone();
        self.adapt(left, result.clone());
        self.emit(Instruction::Return(result));
    }

    pub(crate) fn finish(self) -> Body {
        Body {
            instructions: self.instructions,
            locals: self.next_slot - self.first_local,
            guards: Vec::new(),
        }
    }

    /// Lowers every statement, leaving what the last one leaves and discarding the rest.
    pub(crate) fn block(&mut self, block: &Block) -> Option<Descriptor> {
        let last = block.statements.len().saturating_sub(1);
        let mut left = None;
        for (at, statement) in block.statements.iter().enumerate() {
            left = self.statement(statement);
            if at < last {
                self.adapt(left.take(), None);
            }
        }
        left
    }

    /// A bare expression is the only statement that leaves anything behind.
    fn statement(&mut self, statement: &Statement) -> Option<Descriptor> {
        if let StatementKind::Expr(expr) = &statement.kind {
            return self.expr(expr);
        }
        self.performed(&statement.kind);
        None
    }

    fn performed(&mut self, statement: &StatementKind) {
        match statement {
            StatementKind::Binding { name, value, .. } => self.binding(name, value),
            StatementKind::Assign {
                target,
                operator,
                value,
            } => self.assign(target, *operator, value),
            StatementKind::Return(value) => self.returned(value.as_ref()),
            StatementKind::Break => self.broken(),
            StatementKind::Continue => self.continued(),
            StatementKind::For(repeated) => self.for_loop(repeated),
            StatementKind::Discard(value) => self.discarded(value),
            StatementKind::Expr(_) => {}
        }
    }

    /// `_ = save(user)`: the value is worked out for its effect and then dropped.
    fn discarded(&mut self, value: &Expr) {
        let left = self.expr(value);
        self.adapt(left, None);
    }

    fn binding(&mut self, name: &Name, value: &Expr) {
        if self.splitting.contains(&name.span) {
            self.split(name, value);
            return;
        }
        let left = self.expr(value);
        self.declare(name);
        self.stored(name, left);
    }

    /// A record kept in locals: each field is worked out in the order the type declares them,
    /// which is the order the constructor would have taken them in, and put in a local of its own.
    fn split(&mut self, name: &Name, value: &Expr) {
        let ExprKind::Record { base, fields } = &value.kind else {
            unreachable!("a binding is split only where it holds a record literal")
        };
        for carried in self
            .lowering
            .shapes
            .built(&base.to_string())
            .clone()
            .carries
        {
            self.given(fields, &carried);
            let Some(of) = carried.of else {
                continue;
            };
            let at = self.temporary(&of);
            self.emit(Instruction::Store {
                slot: at,
                of: of.clone(),
            });
            let held = Held {
                binding: name.span,
                field: carried.name,
            };
            self.fields.insert(held, Slot { at, of });
        }
    }

    /// Whether the binding `name` means is one kept in locals, and where it was declared.
    pub(crate) fn split_binding(&self, name: &Name) -> Option<Span> {
        let Origin::Declared(at) = self.definition(name).origin else {
            return None;
        };
        self.splitting.contains(&at).then_some(at)
    }

    fn assign(&mut self, name: &Name, operator: AssignOperator, value: &Expr) {
        let left = match (operator, self.local(name)) {
            (AssignOperator::Add, Some(slot)) => Some(self.added_to(&slot, value)),
            (AssignOperator::Set | AssignOperator::Add, _) => self.expr(value),
        };
        self.stored(name, left);
    }

    fn returned(&mut self, value: Option<&Expr>) {
        let result = self.result.clone();
        if let Some(expr) = value {
            let left = self.expr(expr);
            self.adapt(left, result.clone());
        }
        self.emit(Instruction::Return(result));
    }

    fn broken(&mut self) {
        let done = self.loops.last().map(|repeat| repeat.done);
        self.leave(done);
    }

    fn continued(&mut self) {
        let again = self.loops.last().map(|repeat| repeat.again);
        self.leave(again);
    }

    fn leave(&mut self, repeat: Option<Label>) {
        if let Some(label) = repeat {
            self.emit(Instruction::Jump(label));
        }
    }

    fn for_loop(&mut self, repeated: &ForLoop) {
        let turn = self.entered(&repeated.header);
        let top = self.label();
        let again = self.label();
        let done = self.label();
        self.emit(Instruction::Label(top));
        self.tested(&turn, done);
        self.loops.push(Repeat { again, done });
        let left = self.block(&repeated.body);
        self.adapt(left, None);
        self.loops.pop();
        self.emit(Instruction::Label(again));
        if let Turn::Walk { index, .. } = turn {
            self.emit(Instruction::Increment { slot: index });
        }
        self.emit(Instruction::Jump(top));
        self.emit(Instruction::Label(done));
    }

    /// What the header sets up before the first turn, which only a `for … in` has any of.
    fn entered<'h>(&mut self, header: &'h ForHeader) -> Turn<'h> {
        match header {
            ForHeader::Forever => Turn::Forever,
            ForHeader::While(condition) => Turn::While(condition),
            ForHeader::In { binding, iterable } => {
                let of = Descriptor::reference(LIST);
                let left = self.expr(iterable);
                self.adapt(left, Some(of.clone()));
                let list = self.temporary(&of);
                self.emit(Instruction::Store { slot: list, of });
                let index = self.temporary(&Descriptor::Integer);
                self.emit(Instruction::Integer(0));
                self.emit(Instruction::Store {
                    slot: index,
                    of: Descriptor::Integer,
                });
                self.declare(binding);
                Turn::Walk {
                    list,
                    index,
                    item: self.local(binding),
                }
            }
        }
    }

    /// What one turn tests before it runs the body, and where it goes when the test fails.
    fn tested(&mut self, turn: &Turn, done: Label) {
        match turn {
            Turn::Forever => {}
            Turn::While(condition) => {
                self.condition(condition);
                self.emit(Instruction::JumpIfFalse(done));
            }
            Turn::Walk { .. } => self.walked(turn, done),
        }
    }

    /// A turn of a `for … in`: there is another item, and it is what the name holds.
    fn walked(&mut self, turn: &Turn, done: Label) {
        let Turn::Walk { list, index, item } = turn else {
            return;
        };
        let of = Descriptor::reference(LIST);
        let counting = Instruction::Load {
            slot: *index,
            of: Descriptor::Integer,
        };
        let walking = Instruction::Load {
            slot: *list,
            of: of.clone(),
        };
        self.emit(counting.clone());
        self.emit(walking.clone());
        self.emit(Instruction::InvokeInterface(reaching(
            "size",
            Vec::new(),
            Descriptor::Integer,
        )));
        self.emit(Instruction::CompareIntegers(crate::code::Comparison::Less));
        self.emit(Instruction::JumpIfFalse(done));
        self.emit(walking);
        self.emit(counting);
        self.emit(Instruction::InvokeInterface(reaching(
            "get",
            vec![Descriptor::Integer],
            object(),
        )));
        let held = item.clone();
        self.held(held, Some(object()));
    }

    /// Puts what is on the stack into the slot a name holds, when the name holds one at all.
    pub(crate) fn stored(&mut self, name: &Name, left: Option<Descriptor>) {
        let held = self.local(name);
        self.held(held, left);
    }

    fn held(&mut self, slot: Option<Slot>, left: Option<Descriptor>) {
        let Some(slot) = slot else {
            self.adapt(left, None);
            return;
        };
        self.adapt(left, Some(slot.of.clone()));
        self.emit(Instruction::Store {
            slot: slot.at,
            of: slot.of,
        });
    }

    /// Lowers a truth value, which every condition is and which a jump reads as a word.
    pub(crate) fn condition(&mut self, expr: &Expr) {
        let held = self.expr(expr);
        self.adapt(held, Some(Descriptor::Boolean));
    }

    /// Lowers `expr` and leaves it as the type inference settled on, which is what an operator
    /// works on: a call whose result is a type parameter leaves a reference until it is read.
    pub(crate) fn value(&mut self, expr: &Expr) -> Option<Descriptor> {
        let held = self.expr(expr);
        let settled = self.carried(expr.span);
        self.adapt(held, settled.clone());
        settled
    }

    /// Lowers `expr` as the value something that holds it as `wanted` is handed.
    ///
    /// A `()` is carried by nothing at all, so an expression whose type is `()` leaves nothing
    /// behind. Where the thing being handed it holds a reference, something has to stand there,
    /// and this is the one place it is put there.
    pub(crate) fn handed(&mut self, expr: &Expr, wanted: Option<Descriptor>) {
        let held = self.expr(expr);
        let stands_for_nothing =
            held.is_none() && wanted.is_some() && self.holds_nothing(expr.span);
        let left = if stands_for_nothing {
            Some(self.nothing())
        } else {
            held
        };
        self.adapt(left, wanted);
    }

    /// The value a `()` stands as where a reference is wanted, which nothing reads back out.
    ///
    /// What stands for `()` holds nothing, because `()` holds nothing, and a Lumen value has no
    /// identity for two of them to be told apart by. `docs/specs/codegen.md` states it.
    fn nothing(&mut self) -> Descriptor {
        self.emit(Instruction::New(object_class()));
        self.emit(Instruction::Copy);
        self.emit(Instruction::Construct(MethodRef {
            class: object_class(),
            name: CONSTRUCTOR.to_owned(),
            descriptor: MethodDescriptor::new(Vec::new(), None),
        }));
        object()
    }

    /// Whether what is written at `written` has the type `()`, which nothing carries.
    fn holds_nothing(&self, written: Span) -> bool {
        self.lowering
            .typed
            .type_of(written)
            .is_some_and(|of| matches!(self.at.substituted(of), Type::Unit))
    }

    /// Makes what is on the stack into what is wanted there, which is nothing when they agree.
    pub(crate) fn adapt(&mut self, from: Option<Descriptor>, to: Option<Descriptor>) {
        match (from, to) {
            (Some(held), Some(wanted)) => self.converted(&held, &wanted),
            (Some(held), None) => self.emit(Instruction::Drop(held)),
            // Nothing on the stack where something is wanted is code nothing reaches: a block
            // that left through a `return` leaves the stack to whatever comes after the jump.
            // A `()` also leaves nothing, and `handed` stands a value there before it gets here.
            (None, _) => {}
        }
    }

    fn converted(&mut self, held: &Descriptor, wanted: &Descriptor) {
        if held == wanted {
            return;
        }
        match (boxing(held), boxing(wanted)) {
            (Some(held), None) => self.boxed(&held),
            (None, Some(wanted)) => self.unboxed(&wanted),
            (None, None) => self.narrowed(held, wanted),
            // Two whole numbers or two truth values that differ is not a thing to convert:
            // `Boolean` and `Integer` are the same word, and nothing else pairs off.
            (Some(_), Some(_)) => {}
        }
    }

    /// A whole number or a truth value held as a reference, which a type parameter wants.
    fn boxed(&mut self, held: &Boxing) {
        self.emit(holding(held));
    }

    /// A reference read back as the whole number or truth value inference says it is.
    fn unboxed(&mut self, wanted: &Boxing) {
        self.emit(Instruction::Cast(wanted.class.clone()));
        self.emit(Instruction::InvokeVirtual(MethodRef {
            class: wanted.class.clone(),
            name: wanted.read.to_owned(),
            descriptor: MethodDescriptor::new(Vec::new(), Some(wanted.of.clone())),
        }));
    }

    /// A reference read back as the class it is; anything else is already what is wanted.
    fn narrowed(&mut self, held: &Descriptor, wanted: &Descriptor) {
        let (Descriptor::Reference(held), Descriptor::Reference(wanted)) = (held, wanted) else {
            return;
        };
        if *held == object_class() && *wanted != object_class() {
            self.emit(Instruction::Cast(wanted.clone()));
        }
    }

    /// What the function gives back, which `return` and `?` both answer to.
    pub(crate) fn result(&self) -> Option<Descriptor> {
        self.result.clone()
    }

    /// Gives the binding `name` declares a local of its own, unless it is carried by nothing.
    pub(crate) fn declare(&mut self, name: &Name) {
        let Some(of) = self.carried(name.span) else {
            return;
        };
        let at = self.temporary(&of);
        self.slots.insert(name.span, Slot { at, of });
    }

    /// Where the binding `name` means lives, which is nowhere when it is carried by nothing.
    pub(crate) fn local(&self, name: &Name) -> Option<Slot> {
        let Origin::Declared(at) = self.definition(name).origin else {
            return None;
        };
        self.slots.get(&at).cloned()
    }

    /// The type of what is written at `written`, as the JVM carries it.
    ///
    /// Every type is asked for through here, so a body written for one use of a generic reads
    /// each type parameter as the type that use settled it at.
    pub(crate) fn carried(&self, written: Span) -> Option<Descriptor> {
        let of = self.lowering.typed.type_of(written)?;
        self.lowering.shapes.carried(&self.at.substituted(of))
    }

    /// The method a use of the function declared at `declared` reaches from inside this body.
    pub(crate) fn reaching(&self, declared: Span, at: Span) -> Reaching {
        self.lowering.used(declared, at, &self.at)
    }

    /// What the use this body is being written for settled each of its type parameters at.
    ///
    /// A call inside a generic is written once per set the generic itself was written for, so a
    /// type parameter standing in the type of a call here stands for whatever that set gives it.
    pub(crate) const fn at(&self) -> &Instantiation {
        &self.at
    }

    pub(crate) fn definition(&self, name: &Name) -> Definition {
        self.lowering
            .typed
            .resolved()
            .definition(Namespace::Value, name)
            .expect("name resolution gave every written value a definition")
    }

    /// The resolution every name of this body was written against.
    pub(crate) fn resolved(&self) -> &ResolvedProgram {
        self.lowering.typed.resolved()
    }

    /// A local of the method's own, which a construct keeps a value in while it works.
    pub(crate) fn temporary(&mut self, of: &Descriptor) -> u16 {
        let at = self.next_slot;
        self.next_slot += of.width();
        at
    }

    pub(crate) fn label(&mut self) -> Label {
        let label = Label(self.next_label);
        self.next_label += 1;
        label
    }

    pub(crate) fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

/// What a whole number or a truth value is held in when a reference is wanted instead.
struct Boxing {
    class: ClassName,
    of: Descriptor,
    /// The method that reads the value back out.
    read: &'static str,
}

/// What holds `of` as a reference, and nothing where a reference is what it is already.
pub(crate) fn as_a_reference(of: &Descriptor) -> Option<Instruction> {
    Some(holding(&boxing(of)?))
}

/// The call that holds a whole number or a truth value as the reference standing for it.
fn holding(held: &Boxing) -> Instruction {
    Instruction::InvokeStatic(MethodRef {
        class: held.class.clone(),
        name: "valueOf".to_owned(),
        descriptor: MethodDescriptor::new(
            vec![held.of.clone()],
            Some(Descriptor::Reference(held.class.clone())),
        ),
    })
}

fn boxing(of: &Descriptor) -> Option<Boxing> {
    let (class, read) = match of {
        Descriptor::Long => ("java/lang/Long", "longValue"),
        Descriptor::Boolean => ("java/lang/Boolean", "booleanValue"),
        Descriptor::Integer => ("java/lang/Integer", "intValue"),
        Descriptor::Reference(_) | Descriptor::Array(_) => return None,
    };
    Some(Boxing {
        class: ClassName::new(class),
        of: of.clone(),
        read,
    })
}

/// A method of the list a `for … in` walks.
pub(crate) fn reaching(name: &str, parameters: Vec<Descriptor>, result: Descriptor) -> MethodRef {
    MethodRef {
        class: ClassName::new(LIST),
        name: name.to_owned(),
        descriptor: MethodDescriptor::new(parameters, Some(result)),
    }
}
