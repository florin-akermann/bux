//! What each expression leaves on the stack.

use lumen_ast::{BinaryOperator, Expr, ExprKind, FieldValue, IfExpr, Name, Span, UnaryOperator};
use lumen_resolver::{DefinitionKind, Origin};
use lumen_types::Type;

use crate::code::{Arithmetic, Comparison, FieldRef, Instruction, MethodRef};
use crate::descriptor::Descriptor;
use crate::lower::body::Builder;
use crate::lower::equality::compared;
use crate::lower::shape::{CONSTRUCTOR, Carried, ERR, OK, Shape, TAG, object};

impl Builder<'_> {
    /// Lowers `expr`, leaving its value on the stack when its type is carried by anything.
    pub(crate) fn expr(&mut self, expr: &Expr) -> Option<Descriptor> {
        match &expr.kind {
            ExprKind::Unit => None,
            ExprKind::Integer(value) => {
                Some(self.left(Instruction::Long(*value), Descriptor::Long))
            }
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
            ExprKind::Call { callee, arguments } => self.call(callee, arguments),
            ExprKind::Field { receiver, name } => self.field(receiver, name),
            ExprKind::Try(inner) => self.propagated(inner, expr.span),
            ExprKind::Record { base, fields } => Some(self.record(base, fields)),
            ExprKind::If(chain) => self.if_expr(chain, expr.span),
            ExprKind::Match(matched) => self.match_expr(matched, expr.span),
        }
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
            DefinitionKind::Local | DefinitionKind::Parameter => self.loaded(name),
            DefinitionKind::Function
            | DefinitionKind::Module
            | DefinitionKind::Type
            | DefinitionKind::TypeParameter => {
                unreachable!("version 0.1 reaches a function by calling it and nothing else")
            }
        }
    }

    /// `!` and `-` each work on one settled type, whatever the operand was written as.
    fn unary(&mut self, operator: UnaryOperator, operand: &Expr) -> Descriptor {
        let (of, instruction) = worked_on(operator);
        let held = self.expr(operand);
        self.adapt(held, Some(of.clone()));
        self.emit(instruction);
        of
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
            settled => self.applied(settled, left, right),
        }
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

    /// Both sides are run at the type they were inferred to have, and the operator decides.
    fn applied(
        &mut self,
        operator: BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> Option<Descriptor> {
        let held = self.value(left);
        let other = self.value(right);
        self.adapt(other, held.clone());
        match operator {
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                Some(self.same(operator, held.as_ref()))
            }
            BinaryOperator::Less
            | BinaryOperator::LessOrEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterOrEqual => {
                self.emit(Instruction::CompareLongs(how(operator)));
                Some(Descriptor::Boolean)
            }
            BinaryOperator::Add => self.added(held),
            BinaryOperator::Or | BinaryOperator::And => held,
            arithmetic => {
                self.emit(Instruction::Arithmetic(counted(arithmetic)));
                Some(Descriptor::Long)
            }
        }
    }

    fn same(&mut self, operator: BinaryOperator, held: Option<&Descriptor>) -> Descriptor {
        for instruction in compared(held, how(operator)) {
            self.emit(instruction);
        }
        Descriptor::Boolean
    }

    /// `+` joins two strings and adds two whole numbers, which is what their types say.
    fn added(&mut self, held: Option<Descriptor>) -> Option<Descriptor> {
        match &held {
            Some(Descriptor::Reference(_)) => self.emit(Instruction::Concat),
            _ => self.emit(Instruction::Arithmetic(Arithmetic::Add)),
        }
        held
    }

    fn call(&mut self, callee: &Expr, arguments: &[Expr]) -> Option<Descriptor> {
        let ExprKind::Name(name) = &callee.kind else {
            unreachable!("version 0.1 calls a name, which is a function or a constructor")
        };
        match self.definition(name).kind {
            DefinitionKind::Constructor => Some(self.built(name, arguments)),
            _ => self.invoked(name, arguments),
        }
    }

    /// A call of a function of the module, which is a static method of the module class.
    fn invoked(&mut self, name: &Name, arguments: &[Expr]) -> Option<Descriptor> {
        let Origin::Declared(at) = self.definition(name).origin else {
            unreachable!("version 0.1 has no function the prelude supplies")
        };
        let signature = self.lowering.signature(at);
        for (argument, wanted) in arguments.iter().zip(&signature.parameters) {
            let held = self.expr(argument);
            self.adapt(held, wanted.clone());
        }
        self.emit(Instruction::InvokeStatic(MethodRef {
            class: self.lowering.shapes.module().clone(),
            name: name.text.clone(),
            descriptor: signature.descriptor(),
        }));
        signature.result
    }

    /// Builds what a constructor builds, out of the values it is given in order.
    fn built(&mut self, name: &Name, arguments: &[Expr]) -> Descriptor {
        let shape = self.lowering.shapes.built(&name.text).clone();
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        for (argument, carried) in arguments.iter().zip(&shape.carries) {
            let held = self.expr(argument);
            self.adapt(held, carried.of.clone());
        }
        self.constructed(&shape)
    }

    fn field(&mut self, receiver: &Expr, name: &Name) -> Option<Descriptor> {
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

    /// `f(x)?` gives back the error it was handed, and otherwise reads the value out of it.
    fn propagated(&mut self, inner: &Expr, at: Span) -> Option<Descriptor> {
        let failed = self.lowering.shapes.built(ERR).clone();
        let worked = self.lowering.shapes.built(OK).clone();
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

    fn record(&mut self, base: &Name, fields: &[FieldValue]) -> Descriptor {
        if self.definition(base).kind == DefinitionKind::Constructor {
            return self.built_with(base, fields);
        }
        self.updated(base, fields)
    }

    /// `User { id: id }` builds a record out of a value for each field it declares.
    fn built_with(&mut self, base: &Name, fields: &[FieldValue]) -> Descriptor {
        let shape = self.lowering.shapes.built(&base.text).clone();
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        for carried in &shape.carries {
            let Some(given) = fields.iter().find(|field| field.name.text == carried.name) else {
                unreachable!("inference gave every field of a record a value")
            };
            let held = self.expr(&given.value);
            self.adapt(held, carried.of.clone());
        }
        self.constructed(&shape)
    }

    /// `user { active: false }` builds another one, from the old fields where none is given.
    fn updated(&mut self, base: &Name, fields: &[FieldValue]) -> Descriptor {
        let shape = self.record_updated(base);
        self.emit(Instruction::New(shape.class.clone()));
        self.emit(Instruction::Copy);
        for carried in &shape.carries {
            match fields.iter().find(|field| field.name.text == carried.name) {
                Some(given) => {
                    let held = self.expr(&given.value);
                    self.adapt(held, carried.of.clone());
                }
                None => self.kept(base, &shape, carried),
            }
        }
        self.constructed(&shape)
    }

    /// One field of the record being updated, read off the one the update was written against.
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

/// Which comparison an operator asks for.
fn how(operator: BinaryOperator) -> Comparison {
    match operator {
        BinaryOperator::NotEqual => Comparison::NotEqual,
        BinaryOperator::Less => Comparison::Less,
        BinaryOperator::LessOrEqual => Comparison::LessOrEqual,
        BinaryOperator::Greater => Comparison::Greater,
        BinaryOperator::GreaterOrEqual => Comparison::GreaterOrEqual,
        _ => Comparison::Equal,
    }
}

/// What an operator does to two whole numbers.
fn counted(operator: BinaryOperator) -> Arithmetic {
    match operator {
        BinaryOperator::Subtract => Arithmetic::Subtract,
        BinaryOperator::Multiply => Arithmetic::Multiply,
        BinaryOperator::Divide => Arithmetic::Divide,
        BinaryOperator::Remainder => Arithmetic::Remainder,
        _ => Arithmetic::Add,
    }
}

/// What `!` and `-` each work on, and the instruction that does the work.
fn worked_on(operator: UnaryOperator) -> (Descriptor, Instruction) {
    match operator {
        UnaryOperator::Not => (Descriptor::Boolean, Instruction::Not),
        UnaryOperator::Negate => (
            Descriptor::Long,
            Instruction::Arithmetic(Arithmetic::Negate),
        ),
    }
}
