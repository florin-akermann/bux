//! What each expression leaves on the stack.

use std::iter;

use lumen_ast::{BinaryOperator, Expr, ExprKind, FieldValue, IfExpr, Name, Path};
use lumen_ast::{Span, UnaryOperator};
use lumen_resolver::{DefinitionKind, Origin};
use lumen_types::Type;

use crate::code::{Arithmetic, Comparison, FieldRef, Instruction, MethodRef};
use crate::descriptor::Descriptor;
use crate::lower::Instance;
use crate::lower::body::{Builder, Held, LIST};
use crate::lower::modules::Through;
use crate::lower::operator::is_negation;
use crate::lower::shape::{CONSTRUCTOR, Carried, ERR, NONE, OK, OPTION, RESULT};
use crate::lower::shape::{SOME, Shape, TAG};
use crate::lower::shape::{object, object_class};

impl Builder<'_> {
    /// Lowers `expr`, leaving its value on the stack when its type is carried by anything.
    pub(crate) fn expr(&mut self, expr: &Expr) -> Option<Descriptor> {
        match &expr.kind {
            ExprKind::Unit => None,
            ExprKind::Integer(value) => Some(self.whole_number(*value, expr.span)),
            ExprKind::Bool(value) => {
                Some(self.left(Instruction::Boolean(*value), Descriptor::Boolean))
            }
            ExprKind::String(value) => Some(self.left(
                Instruction::Text(value.clone()),
                Descriptor::reference("java/lang/String"),
            )),
            ExprKind::Name(name) => self.written(name),
            ExprKind::Unary { operator, operand } => Some(self.unary(*operator, operand)),
            ExprKind::Binary {
                operator,
                left,
                right,
            } => self.binary(*operator, left, right),
            ExprKind::Call { callee, arguments } => {
                self.call(callee, &arguments.values(), expr.span)
            }
            ExprKind::Field { receiver, name } => self.field(receiver, name),
            ExprKind::Try(inner) => self.propagated(inner, expr.span),
            ExprKind::List(elements) => Some(self.written_list(elements)),
            ExprKind::Record { base, fields } => Some(self.record(base, fields)),
            ExprKind::If(chain) => self.if_expr(chain, expr.span),
            ExprKind::Match(matched) => self.match_expr(matched, expr.span),
        }
    }

    /// A written list: its elements gathered into an array, and the array made into a list.
    ///
    /// A list holds references, so an element whose type is carried by a whole number or a
    /// truth value is boxed on the way in, which is what `docs/specs/codegen.md` states.
    fn written_list(&mut self, elements: &[Expr]) -> Descriptor {
        self.emit(Instruction::Integer(counting(elements.len())));
        self.emit(Instruction::NewArray(object_class()));
        for (at, element) in elements.iter().enumerate() {
            self.emit(Instruction::Copy);
            self.emit(Instruction::Integer(counting(at)));
            self.handed(element, Some(object()));
            self.emit(Instruction::StoreInArray);
        }
        self.emit(Instruction::CollectList);
        Descriptor::reference(LIST)
    }

    /// A literal: one instruction, and the descriptor it leaves behind.
    fn left(&mut self, instruction: Instruction, of: Descriptor) -> Descriptor {
        self.emit(instruction);
        of
    }

    /// A name means a binding to read, or a constructor that carries nothing to build.
    fn written(&mut self, name: &Name) -> Option<Descriptor> {
        match self.definition(name).kind {
            DefinitionKind::Constructor => Some(self.built(name, &[])),
            DefinitionKind::Local | DefinitionKind::Variable | DefinitionKind::Parameter => {
                self.loaded(name)
            }
            DefinitionKind::Function
            | DefinitionKind::TraitMethod
            | DefinitionKind::Module
            | DefinitionKind::Type
            | DefinitionKind::Trait
            | DefinitionKind::TypeParameter => {
                unreachable!(
                    "`{}` is not a value, and the front end refused it as one",
                    name.text
                )
            }
        }
    }

    /// `!` is `Bool`'s alone, and prefix `-` is `Neg`, which a type gets by writing an instance.
    fn unary(&mut self, operator: UnaryOperator, operand: &Expr) -> Descriptor {
        if is_negation(operator) {
            return self.negated(operand);
        }
        let held = self.expr(operand);
        self.adapt(held, Some(Descriptor::Boolean));
        self.emit(Instruction::Not);
        Descriptor::Boolean
    }

    fn binary(
        &mut self,
        operator: BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> Option<Descriptor> {
        match operator {
            BinaryOperator::Or => Some(self.either(left, right)),
            BinaryOperator::And => Some(self.both(left, right)),
            settled => self.operated(settled, left, right),
        }
    }

    /// `/` and `%` have no answer for a zero divisor, so each leaves an `Option<Int>`.
    ///
    /// `ldiv` and `lrem` throw on a zero divisor and no method a module writes may throw, so the
    /// divisor is tested first and the answer is only ever worked out where there is one.
    pub(crate) fn divided(&mut self, how: Arithmetic, left: &Expr, right: &Expr) -> Descriptor {
        let dividend = self.put_aside(left);
        let divisor = self.put_aside(right);
        self.emit(Instruction::Load {
            slot: divisor,
            of: Descriptor::Long,
        });
        self.emit(Instruction::Long(0));
        self.emit(Instruction::CompareLongs(Comparison::Equal));
        let answered = self.label();
        let end = self.label();
        self.emit(Instruction::JumpIfFalse(answered));
        let of = self.absent();
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(answered));
        self.answer(how, dividend, divisor);
        self.emit(Instruction::Label(end));
        of
    }

    /// One operand of a division, put in a local so that the divisor can be tested on its own.
    pub(crate) fn put_aside(&mut self, operand: &Expr) -> u16 {
        self.put_aside_as(operand, &Descriptor::Long)
    }

    /// Runs `operand` and keeps its value in a local of `of`, to be loaded again below.
    ///
    /// A side of an operator may be a call, so both sides are run in the order they are written
    /// even where the method they are handed to takes them the other way round.
    pub(crate) fn put_aside_as(&mut self, operand: &Expr, of: &Descriptor) -> u16 {
        let held = self.value(operand);
        self.adapt(held, Some(of.clone()));
        let slot = self.temporary(of);
        self.emit(Instruction::Store {
            slot,
            of: of.clone(),
        });
        slot
    }

    /// The `None` a zero divisor gives back, which is the whole of that branch.
    fn absent(&mut self) -> Descriptor {
        let shape = self.lowering.shapes.built(NONE).clone();
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        self.constructed(&shape)
    }

    /// The `Some` a divisor that is not zero gives back, holding the quotient or the remainder.
    fn answer(&mut self, how: Arithmetic, dividend: u16, divisor: u16) {
        let shape = self.lowering.shapes.built(SOME).clone();
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        for slot in [dividend, divisor] {
            self.emit(Instruction::Load {
                slot,
                of: Descriptor::Long,
            });
        }
        self.emit(Instruction::Arithmetic(how));
        let carried = shape.carries.first().map(|held| held.of.clone());
        self.adapt(Some(Descriptor::Long), carried.flatten());
        self.constructed(&shape);
    }

    /// `a || b` leaves `true` without running `b`, which is what short-circuiting is.
    fn either(&mut self, left: &Expr, right: &Expr) -> Descriptor {
        let otherwise = self.label();
        let end = self.label();
        self.condition(left);
        self.emit(Instruction::JumpIfFalse(otherwise));
        self.emit(Instruction::Boolean(true));
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(otherwise));
        self.condition(right);
        self.emit(Instruction::Label(end));
        Descriptor::Boolean
    }

    /// `a && b` leaves `false` without running `b`.
    fn both(&mut self, left: &Expr, right: &Expr) -> Descriptor {
        let otherwise = self.label();
        let end = self.label();
        self.condition(left);
        self.emit(Instruction::JumpIfFalse(otherwise));
        self.condition(right);
        self.emit(Instruction::Jump(end));
        self.emit(Instruction::Label(otherwise));
        self.emit(Instruction::Boolean(false));
        self.emit(Instruction::Label(end));
        Descriptor::Boolean
    }

    /// Where the instance that answers `method` for whatever is written at `of` declares it.
    ///
    /// One of the types the JVM holds has no declaration of its own to reach, even where
    /// `library/prelude.lm` writes the instance: what that instance amounts to is written out
    /// in place rather than called, so there is nothing here.
    pub(crate) fn instance_written(&self, method: &str, of: Span) -> Option<Instance> {
        let at = self.lowering.typed.type_of(of)?;
        self.lowering.answering(method, &self.at().substituted(at))
    }

    /// `+` joins two strings and adds two whole numbers, which is what their types say.
    pub(crate) fn added(&mut self, held: Option<Descriptor>) -> Option<Descriptor> {
        match &held {
            Some(Descriptor::Reference(_)) => self.emit(Instruction::Concat),
            _ => self.emit(Instruction::Arithmetic(Arithmetic::Add)),
        }
        held
    }

    fn call(&mut self, callee: &Expr, arguments: &[&Expr], at: Span) -> Option<Descriptor> {
        if let Some(reached) = self.reached_inside(callee) {
            return self.inside_module(&reached, arguments);
        }
        let (name, passed) = written_as(callee, arguments);
        match self.definition(name).kind {
            DefinitionKind::Constructor => Some(self.built(name, &passed)),
            _ => self.invoked(name, &passed, at),
        }
    }

    /// The module a callee reaches inside, and the name it reaches, where it reaches one.
    fn reached_inside<'w>(&self, callee: &'w Expr) -> Option<Through<'w>> {
        let ExprKind::Field { receiver, name } = &callee.kind else {
            return None;
        };
        let module = self.resolved().module_reached(receiver)?;
        Some(Through {
            module,
            name,
            used: callee.span,
        })
    }

    /// A call of a function of the module, which is a static method of the module class.
    fn invoked(&mut self, name: &Name, arguments: &[&Expr], written: Span) -> Option<Descriptor> {
        if self.definition(name).kind == DefinitionKind::TraitMethod {
            return self.through_the_instance(name, arguments);
        }
        let Origin::Declared(at) = self.definition(name).origin else {
            return self.supplied(name, arguments, written);
        };
        self.statically(at, name.span, arguments)
    }

    /// A call of a trait method, which is a call of the instance its type has.
    ///
    /// Which instance is settled where the call is written, so nothing is looked up while the
    /// program runs; `docs/specs/traits.md` states that a generic is written once per set of
    /// types and that the body so written calls the instances those types have.
    fn through_the_instance(&mut self, name: &Name, arguments: &[&Expr]) -> Option<Descriptor> {
        let at = self
            .lowering
            .typed
            .instance_at(name.span)
            .expect("inference settled the instance every use of a trait method reaches");
        let reached = self
            .lowering
            .answering(&name.text, &self.at().substituted(at));
        match reached {
            Some(instance) => self.calling(&instance, arguments),
            None => Some(self.written_out_instance(name, arguments)),
        }
    }

    /// A call of the method an instance writes, over the values it is handed in order.
    pub(crate) fn calling(
        &mut self,
        instance: &Instance,
        arguments: &[&Expr],
    ) -> Option<Descriptor> {
        for (argument, wanted) in arguments.iter().zip(&instance.signature().parameters) {
            self.handed(argument, wanted.clone());
        }
        self.emit(instance.called());
        instance.signature().result.clone()
    }

    /// A call of the method declared at `at`, which is a static method of the module class.
    pub(crate) fn statically(
        &mut self,
        at: Span,
        used: Span,
        arguments: &[&Expr],
    ) -> Option<Descriptor> {
        let reached = self.reaching(at, used);
        for (argument, wanted) in arguments.iter().zip(&reached.signature.parameters) {
            self.handed(argument, wanted.clone());
        }
        self.emit(Instruction::InvokeStatic(MethodRef {
            class: self.lowering.shapes.module().clone(),
            name: reached.named,
            descriptor: reached.signature.descriptor(),
        }));
        reached.signature.result
    }

    /// Builds what a constructor builds, out of the values it is given in order.
    fn built(&mut self, name: &Name, arguments: &[&Expr]) -> Descriptor {
        let shape = self.lowering.shapes.built(&name.text).clone();
        self.builds(&shape, arguments)
    }

    fn field(&mut self, receiver: &Expr, name: &Name) -> Option<Descriptor> {
        if let Some(shape) = self.offered_variant(receiver, name) {
            return Some(self.builds(&shape, &[]));
        }
        if let ExprKind::Name(held) = &receiver.kind
            && let Some(binding) = self.split_binding(held)
        {
            return self.held_field(binding, receiver.span, name);
        }
        let shape = self.record_of(receiver.span);
        let held = self.expr(receiver);
        self.adapt(held, Some(Descriptor::Reference(shape.base.clone())));
        let Some(carried) = shape.carries.iter().find(|held| held.name == name.text) else {
            unreachable!("inference gave every field a record that declares it")
        };
        let Some(of) = carried.of.clone() else {
            self.adapt(Some(Descriptor::Reference(shape.base)), None);
            return None;
        };
        self.emit(Instruction::GetField(FieldRef {
            class: shape.class,
            name: name.text.clone(),
            of: of.clone(),
        }));
        Some(of)
    }

    /// The variant another module offers as `name`, where that is what is reached rather than
    /// a field of a record.
    ///
    /// One carrying nothing is written as the name alone, so it is reached here rather than as
    /// the call `docs/specs/modules.md` writes a variant carrying something as.
    fn offered_variant(&self, receiver: &Expr, name: &Name) -> Option<Shape> {
        let ExprKind::Name(module) = &receiver.kind else {
            return None;
        };
        if self.definition(module).kind != DefinitionKind::Module {
            return None;
        }
        self.lowering
            .shapes
            .offered(&module.text, &name.text)
            .cloned()
    }

    /// One field of a binding kept in locals, read from the local it lives in.
    ///
    /// A field carried by nothing lives nowhere and reads as nothing, the way a binding of a unit
    /// value does, so there is no local to read and nothing is emitted. Every other field of the
    /// record is in one, because the binding put it there when it was split.
    fn held_field(&mut self, binding: Span, at: Span, name: &Name) -> Option<Descriptor> {
        let shape = self.record_of(at);
        let Some(carried) = shape.carries.iter().find(|held| held.name == name.text) else {
            unreachable!("inference gave every field a record that declares it")
        };
        carried.of.as_ref()?;
        let held = Held {
            binding,
            field: name.text.clone(),
        };
        let Some(slot) = self.fields.get(&held).cloned() else {
            unreachable!("splitting a binding put every field it carries in a local")
        };
        self.emit(Instruction::Load {
            slot: slot.at,
            of: slot.of.clone(),
        });
        Some(slot.of)
    }

    /// `?` hands the case with nothing to go on back, and reads the value out of the other.
    ///
    /// The two kinds lower the same way: a test of the tag, the value handed back as the
    /// function's answer where it is the case that propagates, and the value read out of it
    /// where it is not. `docs/specs/arithmetic.md` states it for an `Option`, whose `None` goes
    /// back exactly as it was handed over, carrying nothing to be built again.
    fn propagated(&mut self, inner: &Expr, at: Span) -> Option<Descriptor> {
        let (handed_back, carrying) = self.propagates(inner.span);
        let failed = self.lowering.shapes.built(handed_back).clone();
        let worked = self.lowering.shapes.built(carrying).clone();
        let of = Descriptor::Reference(failed.base.clone());
        let held = self.expr(inner);
        self.adapt(held, Some(of.clone()));
        let slot = self.temporary(&of);
        self.emit(Instruction::Store {
            slot,
            of: of.clone(),
        });
        let value = self.label();
        self.emit(Instruction::Load {
            slot,
            of: of.clone(),
        });
        self.emit(Instruction::GetField(FieldRef {
            class: failed.base.clone(),
            name: TAG.to_owned(),
            of: Descriptor::Integer,
        }));
        self.emit(Instruction::Integer(failed.tag.unwrap_or_default()));
        self.emit(Instruction::CompareIntegers(Comparison::Equal));
        self.emit(Instruction::JumpIfFalse(value));
        self.emit(Instruction::Load {
            slot,
            of: of.clone(),
        });
        self.returning();
        self.emit(Instruction::Label(value));
        self.emit(Instruction::Load { slot, of });
        self.emit(Instruction::Cast(worked.class.clone()));
        self.emit(Instruction::GetField(FieldRef {
            class: worked.class,
            name: carried_name(&worked.carries),
            of: object(),
        }));
        let wanted = self.carried(at);
        self.adapt(Some(object()), wanted.clone());
        wanted
    }

    fn record(&mut self, base: &Path, fields: &[FieldValue]) -> Descriptor {
        if base.module.is_some() || self.definition(&base.name).kind == DefinitionKind::Constructor
        {
            return self.built_with(base, fields);
        }
        self.updated(&base.name, fields)
    }

    /// `User { id: id }` builds a record out of a value for each field it declares.
    fn built_with(&mut self, base: &Path, fields: &[FieldValue]) -> Descriptor {
        let shape = self.lowering.shapes.built(&base.to_string()).clone();
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        for carried in &shape.carries {
            self.given(fields, carried);
        }
        self.constructed(&shape)
    }

    /// Builds one value of `shape`, out of the values the constructor is given in order.
    pub(crate) fn builds(&mut self, shape: &Shape, arguments: &[&Expr]) -> Descriptor {
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        for (argument, carried) in arguments.iter().zip(&shape.carries) {
            self.handed(argument, carried.of.clone());
        }
        self.constructed(shape)
    }

    /// The value the literal gives the field `carried`, which inference proved it gives.
    pub(crate) fn given(&mut self, fields: &[FieldValue], carried: &Carried) {
        let Some(given) = written_for(fields, carried) else {
            unreachable!("inference gave every field of a record a value")
        };
        self.handed(&given.value, carried.of.clone());
    }

    /// `user { active: false }` builds another one, from the old fields where none is given.
    fn updated(&mut self, base: &Name, fields: &[FieldValue]) -> Descriptor {
        let shape = self.record_updated(base);
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        for carried in &shape.carries {
            match written_for(fields, carried) {
                Some(given) => self.handed(&given.value, carried.of.clone()),
                None => self.kept(base, &shape, carried),
            }
        }
        self.constructed(&shape)
    }

    /// One field of the record being updated, read off the one the update was written against.
    ///
    /// A field carried by nothing is kept by doing nothing: a field typed `()` holds no value to
    /// read, and the constructor the update calls does not take one for it either.
    fn kept(&mut self, base: &Name, shape: &Shape, carried: &Carried) {
        let Some(of) = carried.of.clone() else {
            return;
        };
        let held = self.loaded(base);
        self.adapt(held, Some(Descriptor::Reference(shape.class.clone())));
        self.emit(Instruction::GetField(FieldRef {
            class: shape.class.clone(),
            name: carried.name.clone(),
            of,
        }));
    }

    fn if_expr(&mut self, chain: &IfExpr, at: Span) -> Option<Descriptor> {
        let wanted = self.carried(at);
        let end = self.label();
        for branch in &chain.branches {
            let otherwise = self.label();
            self.condition(&branch.condition);
            self.emit(Instruction::JumpIfFalse(otherwise));
            let held = self.block(&branch.block);
            self.adapt(held, wanted.clone());
            self.emit(Instruction::Jump(end));
            self.emit(Instruction::Label(otherwise));
        }
        if let Some(block) = &chain.otherwise {
            let held = self.block(block);
            self.adapt(held, wanted.clone());
        }
        self.emit(Instruction::Label(end));
        wanted
    }

    /// Leaves the method with what is on the stack, adapted to what the function gives back.
    fn returning(&mut self) {
        let result = self.result();
        self.emit(Instruction::Return(result));
    }

    /// The record the binding an update is written against holds, which is what it rebuilds.
    ///
    /// The name is the base of the update rather than an expression of its own, so its type is
    /// the one it was declared with rather than one inference recorded where it is written.
    fn record_updated(&self, base: &Name) -> Shape {
        let Origin::Declared(at) = self.definition(base).origin else {
            unreachable!("a record is updated through a binding, which is declared where it is")
        };
        self.record_of(at)
    }

    /// The variant a `?` at `written` hands back, and the one it reads its value out of.
    fn propagates(&self, written: Span) -> (&'static str, &'static str) {
        let Some(Type::Named { name, .. }) = self.lowering.typed.type_of(written) else {
            unreachable!("inference gave every `?` a type to propagate")
        };
        match name.as_str() {
            OPTION => (NONE, SOME),
            RESULT => (ERR, OK),
            other => unreachable!("`?` propagates an `Option` or a `Result`, never a `{other}`"),
        }
    }

    /// The record the type of what is written at `written` is, which is what carries its fields.
    pub(crate) fn record_of(&self, written: Span) -> Shape {
        let Some(Type::Named { name, .. }) = self.lowering.typed.type_of(written) else {
            unreachable!("a field is reached only through a record type")
        };
        self.lowering.shapes.record(name).clone()
    }

    pub(crate) fn constructed(&mut self, shape: &Shape) -> Descriptor {
        self.emit(Instruction::Construct(MethodRef {
            class: shape.class.clone(),
            name: CONSTRUCTOR.to_owned(),
            descriptor: shape.constructor(),
        }));
        Descriptor::Reference(shape.base.clone())
    }

    /// Reads what a binding holds, which is nothing when it is carried by nothing.
    pub(crate) fn loaded(&mut self, name: &Name) -> Option<Descriptor> {
        let slot = self.local(name)?;
        self.emit(Instruction::Load {
            slot: slot.at,
            of: slot.of.clone(),
        });
        Some(slot.of)
    }
}

/// The one value an `Ok` carries, which is what `?` reads out of it.
fn carried_name(carries: &[Carried]) -> String {
    carries
        .first()
        .map(|carried| carried.name.clone())
        .unwrap_or_default()
}

/// The value a record literal writes for the field `carried`, where it writes one.
fn written_for<'a>(fields: &'a [FieldValue], carried: &Carried) -> Option<&'a FieldValue> {
    fields.iter().find(|field| field.name.text == carried.name)
}

/// How many, as the JVM counts an array's length and an index into one.
///
/// A source file long enough to write more elements than that is one no filesystem holds, and
/// clamping it would write an array of the wrong length rather than say so.
fn counting(many: usize) -> i32 {
    i32::try_from(many).expect("a written list is shorter than a file can hold")
}

/// The name a call calls, and the arguments it passes, which the receiver begins where one is
/// written in front of the name.
///
/// `docs/specs/calls.md` makes `first.f(rest)` the call `f(first, rest)`, so what is lowered is
/// that call: the same function, the same arguments, and the same order.
fn written_as<'w>(callee: &'w Expr, arguments: &[&'w Expr]) -> (&'w Name, Vec<&'w Expr>) {
    match &callee.kind {
        ExprKind::Name(name) => (name, arguments.to_vec()),
        ExprKind::Field { receiver, name } => (
            name,
            iter::once(receiver.as_ref())
                .chain(arguments.iter().copied())
                .collect(),
        ),
        _ => unreachable!("version 0.1 calls a name, which is a function or a constructor"),
    }
}
