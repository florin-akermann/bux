//! What each instruction becomes, and what it leaves on the stack.

use lumen_ir::{Arithmetic, Comparison, Descriptor, FieldRef, Instruction, Label, MethodRef};

use crate::code::{Assembling, Context};
use crate::frame::Held;
use crate::opcode;

impl Assembling {
    /// Writes `instruction`, and follows what it does to the stack and the locals.
    pub(crate) fn write(&mut self, instruction: &Instruction, context: &mut Context<'_>) {
        match instruction {
            Instruction::Label(_) => {}
            Instruction::Long(value) => self.long(*value, context),
            Instruction::Boolean(held) => self.integer(i32::from(*held), context),
            Instruction::Integer(value) => self.integer(*value, context),
            Instruction::Text(value) => self.text(value, context),
            Instruction::Load { slot, of } => self.load(*slot, of),
            Instruction::Store { slot, of } => self.store(*slot, of),
            Instruction::Drop(of) => self.drop_one(of),
            Instruction::Copy => self.copy(),
            Instruction::Arithmetic(what) => self.arithmetic(*what),
            Instruction::Concat => self.concat(context),
            Instruction::CompareLongs(how) => self.compare_longs(*how, context),
            Instruction::CompareIntegers(how) => self.compare_integers(*how, context),
            Instruction::Not => self.not(),
            Instruction::Widen => self.widen(),
            Instruction::Narrow => self.narrow(),
            Instruction::Jump(label) => self.jump(*label, context),
            Instruction::JumpIfFalse(label) => self.jump_if_false(*label, context),
            Instruction::JumpIfNull(label) => self.jump_if_null(*label, context),
            Instruction::New(class) => self.new_instance(class, context),
            Instruction::Construct(method) => self.construct(method, context),
            Instruction::GetField(field) => self.get_field(field, context),
            Instruction::GetStatic(field) => self.get_static(field, context),
            Instruction::PutField(field) => self.put_field(field, context),
            Instruction::InvokeStatic(method) => self.invoke_static(method, context),
            Instruction::InvokeVirtual(method) => self.invoke_virtual(method, context),
            Instruction::InvokeInterface(method) => self.invoke_interface(method, context),
            Instruction::NewArray(class) => self.new_array(class, context),
            Instruction::StoreInArray => self.store_in_array(),
            Instruction::CollectList => self.collect_list(context),
            Instruction::Cast(class) => self.cast(class, context),
            Instruction::InstanceOf(class) => self.instance_of(class, context),
            Instruction::Increment { slot } => self.increment(*slot),
            Instruction::Return(of) => self.leave(of.as_ref()),
            Instruction::Throw => self.throw(),
        }
    }

    fn long(&mut self, value: i64, context: &mut Context<'_>) {
        let held = context.pool.long(value);
        self.byte(opcode::LDC2_W);
        self.short(held);
        self.push(Held::Long);
    }

    fn integer(&mut self, value: i32, context: &mut Context<'_>) {
        match (i8::try_from(value), i16::try_from(value)) {
            _ if value == 0 => self.byte(opcode::ICONST_0),
            _ if value == 1 => self.byte(opcode::ICONST_1),
            (Ok(narrow), _) => {
                self.byte(opcode::BIPUSH);
                self.byte(narrow.cast_unsigned());
            }
            (_, Ok(short)) => {
                self.byte(opcode::SIPUSH);
                self.short(short.cast_unsigned());
            }
            _ => self.constant(context.pool.integer(value)),
        }
        self.push(Held::Integer);
    }

    fn text(&mut self, value: &str, context: &mut Context<'_>) {
        let held = context.pool.text(value);
        self.constant(held);
        self.push(Held::of(&Descriptor::reference("java/lang/String")));
    }

    /// Pushes what the pool holds at `held`, which is one word wide wherever this is used.
    fn constant(&mut self, held: u16) {
        if let Ok(narrow) = u8::try_from(held) {
            self.byte(opcode::LDC);
            self.byte(narrow);
        } else {
            self.byte(opcode::LDC_W);
            self.short(held);
        }
    }

    fn load(&mut self, slot: u16, of: &Descriptor) {
        let opcode = match of {
            Descriptor::Long => opcode::LLOAD,
            Descriptor::Boolean | Descriptor::Integer | Descriptor::Character => opcode::ILOAD,
            Descriptor::Reference(_) | Descriptor::Array(_) => opcode::ALOAD,
        };
        self.indexed(opcode, slot);
        self.push(Held::of(of));
    }

    fn store(&mut self, slot: u16, of: &Descriptor) {
        let opcode = match of {
            Descriptor::Long => opcode::LSTORE,
            Descriptor::Boolean | Descriptor::Integer | Descriptor::Character => opcode::ISTORE,
            Descriptor::Reference(_) | Descriptor::Array(_) => opcode::ASTORE,
        };
        self.indexed(opcode, slot);
        self.pop();
        self.hold(slot, of);
    }

    fn drop_one(&mut self, of: &Descriptor) {
        self.byte(if of.is_wide() {
            opcode::POP2
        } else {
            opcode::POP
        });
        self.pop();
    }

    fn copy(&mut self) {
        let Some(top) = self.top() else {
            return;
        };
        self.byte(if top.is_wide() {
            opcode::DUP2
        } else {
            opcode::DUP
        });
        self.push(top);
    }

    fn arithmetic(&mut self, what: Arithmetic) {
        let opcode = match what {
            Arithmetic::Add => opcode::LADD,
            Arithmetic::Subtract => opcode::LSUB,
            Arithmetic::Multiply => opcode::LMUL,
            Arithmetic::Divide => opcode::LDIV,
            Arithmetic::Remainder => opcode::LREM,
            Arithmetic::Negate => opcode::LNEG,
        };
        self.byte(opcode);
        if what != Arithmetic::Negate {
            self.pop();
        }
    }

    fn concat(&mut self, context: &mut Context<'_>) {
        let joining = MethodRef {
            class: lumen_ir::ClassName::new("java/lang/String"),
            name: "concat".to_owned(),
            descriptor: lumen_ir::MethodDescriptor::new(
                vec![Descriptor::reference("java/lang/String")],
                Some(Descriptor::reference("java/lang/String")),
            ),
        };
        self.invoke_virtual(&joining, context);
    }

    fn compare_longs(&mut self, how: Comparison, context: &mut Context<'_>) {
        self.byte(opcode::LCMP);
        self.pop();
        self.pop();
        self.push(Held::Integer);
        self.truth(opcode::IFEQ + opcode::step(how), 1, context);
    }

    fn compare_integers(&mut self, how: Comparison, context: &mut Context<'_>) {
        self.truth(opcode::IF_ICMPEQ + opcode::step(how), 2, context);
    }

    /// A small whole number becomes a whole number, which takes the one slot a long takes.
    fn widen(&mut self) {
        self.byte(opcode::I2L);
        self.pop();
        self.push(Held::Long);
    }

    /// A whole number becomes a small one, which the range was read for before it got here.
    fn narrow(&mut self) {
        self.byte(opcode::L2I);
        self.pop();
        self.push(Held::Integer);
    }

    fn not(&mut self) {
        self.byte(opcode::ICONST_1);
        self.push(Held::Integer);
        self.byte(opcode::IXOR);
        self.pop();
    }

    fn new_instance(&mut self, class: &lumen_ir::ClassName, context: &mut Context<'_>) {
        let named = context.pool.class(class);
        let at = self.offset();
        self.byte(opcode::NEW);
        self.short(named);
        self.push(Held::Uninitialised(at));
    }

    /// An array as long as the number on the stack, which the length replaces.
    fn new_array(&mut self, class: &lumen_ir::ClassName, context: &mut Context<'_>) {
        let named = context.pool.class(class);
        self.byte(opcode::ANEWARRAY);
        self.short(named);
        self.pop();
        self.push(Held::of(&Descriptor::array(Descriptor::Reference(
            class.clone(),
        ))));
    }

    /// One element into the array, which takes the array, the index, and the value.
    fn store_in_array(&mut self) {
        self.byte(opcode::AASTORE);
        self.pop();
        self.pop();
        self.pop();
    }

    /// The list of what the array holds, which `java.util.List` declares as a static method.
    ///
    /// It is declared on an interface, so the pool names it as an interface method; that is
    /// what `invokestatic` needs to reach one, and it is the only place a module does.
    fn collect_list(&mut self, context: &mut Context<'_>) {
        let gathering = MethodRef {
            class: lumen_ir::ClassName::new("java/util/List"),
            name: "of".to_owned(),
            descriptor: lumen_ir::MethodDescriptor::new(
                vec![Descriptor::array(Descriptor::reference("java/lang/Object"))],
                Some(Descriptor::reference("java/util/List")),
            ),
        };
        let named = context.pool.interface_method(&gathering);
        self.byte(opcode::INVOKESTATIC);
        self.short(named);
        self.called(&gathering, 0);
    }

    fn construct(&mut self, method: &MethodRef, context: &mut Context<'_>) {
        let named = context.pool.method(method);
        self.byte(opcode::INVOKESPECIAL);
        self.short(named);
        for _ in &method.descriptor.parameters {
            self.pop();
        }
        if let Some(Held::Uninitialised(at)) = self.pop() {
            self.initialised(at, &method.class);
        }
    }

    fn get_field(&mut self, field: &FieldRef, context: &mut Context<'_>) {
        let named = context.pool.field(field);
        self.byte(opcode::GETFIELD);
        self.short(named);
        self.pop();
        self.push(Held::of(&field.of));
    }

    /// A field of a class, which stands on nothing, so nothing comes off the stack for it.
    fn get_static(&mut self, field: &FieldRef, context: &mut Context<'_>) {
        let named = context.pool.field(field);
        self.byte(opcode::GETSTATIC);
        self.short(named);
        self.push(Held::of(&field.of));
    }

    fn put_field(&mut self, field: &FieldRef, context: &mut Context<'_>) {
        let named = context.pool.field(field);
        self.byte(opcode::PUTFIELD);
        self.short(named);
        self.pop();
        self.pop();
    }

    fn invoke_static(&mut self, method: &MethodRef, context: &mut Context<'_>) {
        let named = context.pool.method(method);
        self.byte(opcode::INVOKESTATIC);
        self.short(named);
        self.called(method, 0);
    }

    fn invoke_virtual(&mut self, method: &MethodRef, context: &mut Context<'_>) {
        let named = context.pool.method(method);
        self.byte(opcode::INVOKEVIRTUAL);
        self.short(named);
        self.called(method, 1);
    }

    fn invoke_interface(&mut self, method: &MethodRef, context: &mut Context<'_>) {
        let named = context.pool.interface_method(method);
        let taken: u16 = method
            .descriptor
            .parameters
            .iter()
            .map(Descriptor::width)
            .sum();
        self.byte(opcode::INVOKEINTERFACE);
        self.short(named);
        self.byte(u8::try_from(taken + 1).unwrap_or(u8::MAX));
        self.byte(0);
        self.called(method, 1);
    }

    /// What a call leaves: its arguments gone, the receiver too, and its result on top.
    fn called(&mut self, method: &MethodRef, receivers: usize) {
        for _ in 0..method.descriptor.parameters.len() + receivers {
            self.pop();
        }
        if let Some(result) = &method.descriptor.result {
            self.push(Held::of(result));
        }
    }

    fn instance_of(&mut self, class: &lumen_ir::ClassName, context: &mut Context<'_>) {
        let named = context.pool.class(class);
        self.byte(opcode::INSTANCEOF);
        self.short(named);
        self.pop();
        self.push(Held::Integer);
    }

    /// The count a `for … in` keeps is a local of its own, so this leaves the stack alone.
    fn increment(&mut self, slot: u16) {
        if let Ok(narrow) = u8::try_from(slot) {
            self.byte(opcode::IINC);
            self.byte(narrow);
            self.byte(1);
        } else {
            self.byte(opcode::WIDE);
            self.byte(opcode::IINC);
            self.short(slot);
            self.short(1);
        }
    }

    fn cast(&mut self, class: &lumen_ir::ClassName, context: &mut Context<'_>) {
        let named = context.pool.class(class);
        self.byte(opcode::CHECKCAST);
        self.short(named);
        self.pop();
        self.push(Held::Object(class.clone()));
    }

    fn leave(&mut self, of: Option<&Descriptor>) {
        let opcode = match of {
            None => opcode::RETURN,
            Some(Descriptor::Long) => opcode::LRETURN,
            Some(Descriptor::Boolean | Descriptor::Integer | Descriptor::Character) => {
                opcode::IRETURN
            }
            Some(Descriptor::Reference(_) | Descriptor::Array(_)) => opcode::ARETURN,
        };
        self.byte(opcode);
        self.unreachable();
    }

    fn throw(&mut self) {
        self.byte(opcode::ATHROW);
        self.pop();
        self.unreachable();
    }

    fn jump(&mut self, label: Label, context: &mut Context<'_>) {
        self.branch(opcode::GOTO, label, context);
        self.unreachable();
    }

    fn jump_if_false(&mut self, label: Label, context: &mut Context<'_>) {
        self.pop();
        self.branch(opcode::IFEQ, label, context);
    }

    fn jump_if_null(&mut self, label: Label, context: &mut Context<'_>) {
        self.pop();
        self.branch(opcode::IFNULL, label, context);
    }
}
