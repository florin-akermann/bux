//! The body of the static method an `extern` declaration becomes.
//!
//! `docs/specs/interop.md` states it: the member the declaration names, and the mapping around
//! it. A `field` reads a static field, a `static` calls a static method, a `method` loads the
//! receiver the first parameter is and calls an instance method, and a `new` builds a class and
//! calls its constructor. `Option` reads what came back for `null`, and `Result` guards the call.
//!
//! Every one of these is a method of its own, which is what makes a guard writable: a guarded
//! span begins with an empty stack, and a call may be written wherever an expression is.

use lumen_ast::{Called, ExternDeclaration, Gives, JavaName, Reaches};
use lumen_types::Type;

use crate::code::{Body, FieldRef, Guard, Instruction, Label, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::body::as_a_reference;
use crate::lower::shape::{CONSTRUCTOR, ERR, NONE, OK, OPTION, RESULT, SOME};
use crate::lower::shape::{Shape, Shapes};
use crate::lower::{Lowering, Signature};

/// Everything a JVM can throw, which is the whole of what a `Result` guards against.
const THROWABLE: &str = "java/lang/Throwable";

/// What a throwable is asked to say about itself, which is what the `Err` holds.
const SAYS_OF_ITSELF: &str = "toString";

/// The first instruction the guard covers, which is the first of the body.
const OPENED: Label = Label(0);

/// The instruction after the last one the guard covers, which is where the mapping begins.
const CLOSED: Label = Label(1);

/// Where a throw lands, with the throwable and nothing else on the stack.
const CAUGHT: Label = Label(2);

/// Where a `null` the member gave back lands, which is the `None` it becomes.
const EMPTY: Label = Label(3);

/// What `declaration` runs: the member it names, and the mapping the declaration asks for.
pub(crate) fn body(
    lowering: &Lowering<'_>,
    declaration: &ExternDeclaration,
    signature: &Signature,
) -> Body {
    let shapes = &lowering.shapes;
    let declared = lowering.used_as(declaration.name.span);
    let answer = Answer::of(shapes, declared, declaration.gives);
    let called = shapes.called(received_by(declared));
    let held = signature
        .parameters
        .iter()
        .flatten()
        .map(Descriptor::width)
        .sum();
    let taken: Vec<Descriptor> = signature.parameters.iter().flatten().cloned().collect();
    let mut instructions = vec![Instruction::Label(OPENED)];
    instructions.extend(reached(
        &declaration.reaches,
        &taken,
        answer.member.as_ref(),
        called,
    ));
    instructions.push(Instruction::Label(CLOSED));
    instructions.extend(answer.widening());
    instructions.extend(given_back(shapes, &answer, held));
    if answer.guarded {
        instructions.push(Instruction::Label(CAUGHT));
        instructions.extend(caught(shapes, held));
    }
    Body {
        instructions,
        locals: answer.widened.as_ref().map_or(1, Descriptor::width).max(1),
        guards: guards(answer.guarded),
    }
}

/// What the member gives back, and what the declaration wraps that in before handing it on.
struct Answer {
    /// What the member's own descriptor gives back, which a `void` member makes nothing at all.
    member: Option<Descriptor>,
    /// What stands on the stack once that answer is the type the signature declared, which is
    /// the member's own except where an `int` is widened to the `Int` written beside it.
    widened: Option<Descriptor>,
    /// Whether a `null` the member gives back is a `None` rather than a `Some` of it.
    optional: bool,
    /// Whether the call is guarded, and whatever it throws becomes an `Err`.
    guarded: bool,
}

impl Answer {
    /// The mapping `result` asks for, read off the wrappers it is written with.
    fn of(shapes: &Shapes, declared: &Type, gives: Gives) -> Self {
        let Type::Function { result, .. } = declared else {
            unreachable!("an extern is bound to the function type its signature gives it")
        };
        let (guarded, held) = peeled(RESULT, result);
        let (optional, held) = peeled(OPTION, held);
        let widened = shapes.carried(held);
        Self {
            member: reached_for(widened.as_ref(), gives),
            widened,
            optional,
            guarded,
        }
    }

    /// What turns the member's own answer into the one the signature declared, where it differs.
    fn widening(&self) -> Option<Instruction> {
        (self.member != self.widened).then_some(Instruction::Widen)
    }
}

/// The descriptor the member is reached for, which `int` says is narrower than the result is.
fn reached_for(widened: Option<&Descriptor>, gives: Gives) -> Option<Descriptor> {
    match gives {
        Gives::WhatTheResultIs => widened.cloned(),
        Gives::AnInt => widened.map(|_| Descriptor::Integer),
    }
}

/// What `wrapper` wraps where `held` is one, and `held` itself where it is not.
fn peeled<'a>(wrapper: &str, held: &'a Type) -> (bool, &'a Type) {
    match held {
        Type::Named { name, arguments } if name == wrapper => match arguments.as_slice() {
            [inside, ..] => (true, inside),
            [] => (false, held),
        },
        _ => (false, held),
    }
}

/// The span of the body a `Result` guards, which is the member access and nothing else.
fn guards(guarded: bool) -> Vec<Guard> {
    if !guarded {
        return Vec::new();
    }
    vec![Guard {
        from: OPENED,
        to: CLOSED,
        handler: CAUGHT,
        catching: ClassName::new(THROWABLE),
    }]
}

/// The type the first parameter is, which is the receiver of a `method` and nothing elsewhere.
fn received_by(declared: &Type) -> &Type {
    let Type::Function { parameters, .. } = declared else {
        unreachable!("an extern is bound to the function type its signature gives it")
    };
    parameters.first().unwrap_or(&Type::Unit)
}

/// The member itself: every parameter loaded from the slot it arrived in, and the access.
fn reached(
    reaches: &Reaches,
    taken: &[Descriptor],
    gives: Option<&Descriptor>,
    called: Called,
) -> Vec<Instruction> {
    match reaches {
        Reaches::Field(named) => vec![Instruction::GetStatic(FieldRef {
            class: class_of(named),
            name: member_of(named),
            of: gives.cloned().expect("a field gives back what it holds"),
        })],
        Reaches::Static(named) => {
            let member = MethodRef {
                class: class_of(named),
                name: member_of(named),
                descriptor: MethodDescriptor::new(taken.to_vec(), gives.cloned()),
            };
            with(loaded(taken, 0), Instruction::InvokeStatic(member))
        }
        Reaches::Method(named) => {
            let [receiver, rest @ ..] = taken else {
                unreachable!("an extern method takes the receiver its class is read off")
            };
            let member = MethodRef {
                class: class_named_by(receiver),
                name: named.text.clone(),
                descriptor: MethodDescriptor::new(rest.to_vec(), gives.cloned()),
            };
            with(loaded(taken, 0), calling(called, member))
        }
        Reaches::New => {
            let class = class_named_by(gives.expect("a constructor gives back what it built"));
            let built = MethodRef {
                class: class.clone(),
                name: CONSTRUCTOR.to_owned(),
                descriptor: MethodDescriptor::new(taken.to_vec(), None),
            };
            let mut instructions = vec![Instruction::New(class), Instruction::Copy];
            instructions.extend(loaded(taken, 0));
            with(instructions, Instruction::Construct(built))
        }
    }
}

/// The call `method` is reached with, which the JVM writes one way for an interface's method.
fn calling(called: Called, method: MethodRef) -> Instruction {
    match called {
        Called::AsAClass => Instruction::InvokeVirtual(method),
        Called::AsAnInterface => Instruction::InvokeInterface(method),
    }
}

/// Every parameter loaded, each from the slot it arrives in, in the order they are written.
fn loaded(taken: &[Descriptor], first: u16) -> Vec<Instruction> {
    let mut slot = first;
    let mut instructions = Vec::new();
    for of in taken {
        instructions.push(Instruction::Load {
            slot,
            of: of.clone(),
        });
        slot += of.width();
    }
    instructions
}

/// What the method gives back, once the member has left its own answer on the stack.
///
/// Each way out gives back on its own rather than meeting the others, because two variants of
/// one type are two classes and nothing here needs them to meet.
fn given_back(shapes: &Shapes, answer: &Answer, held: u16) -> Vec<Instruction> {
    let Some(of) = answer.widened.clone().filter(|_| answer.optional) else {
        return handed_on(shapes, answer, answer.widened.clone(), held);
    };
    let mut instructions = vec![
        Instruction::Store {
            slot: held,
            of: of.clone(),
        },
        Instruction::Load {
            slot: held,
            of: of.clone(),
        },
        Instruction::JumpIfNull(EMPTY),
    ];
    let option = Descriptor::Reference(shapes.built(SOME).base.clone());
    instructions.extend(building(shapes.built(SOME), Some((held, of))));
    instructions.extend(handed_on(shapes, answer, Some(option.clone()), held));
    instructions.push(Instruction::Label(EMPTY));
    instructions.extend(building(shapes.built(NONE), None));
    instructions.extend(handed_on(shapes, answer, Some(option), held));
    instructions
}

/// What is on the stack given back, wrapped in `Ok` where the declaration guards the call.
fn handed_on(
    shapes: &Shapes,
    answer: &Answer,
    on_the_stack: Option<Descriptor>,
    held: u16,
) -> Vec<Instruction> {
    if !answer.guarded {
        return vec![Instruction::Return(on_the_stack)];
    }
    let of = on_the_stack.expect("a `Result` wraps a value the member gives back");
    let ok = shapes.built(OK);
    let mut instructions = vec![Instruction::Store {
        slot: held,
        of: of.clone(),
    }];
    instructions.extend(building(ok, Some((held, of))));
    instructions.push(Instruction::Return(Some(Descriptor::Reference(
        ok.base.clone(),
    ))));
    instructions
}

/// What a throw becomes: the `Err` holding what the throwable says of itself, given back.
fn caught(shapes: &Shapes, held: u16) -> Vec<Instruction> {
    let text = Descriptor::reference("java/lang/String");
    let err = shapes.built(ERR);
    let mut instructions = vec![
        Instruction::InvokeVirtual(MethodRef {
            class: ClassName::new(THROWABLE),
            name: SAYS_OF_ITSELF.to_owned(),
            descriptor: MethodDescriptor::new(Vec::new(), Some(text.clone())),
        }),
        Instruction::Store {
            slot: held,
            of: text.clone(),
        },
    ];
    instructions.extend(building(err, Some((held, text))));
    instructions.push(Instruction::Return(Some(Descriptor::Reference(
        err.base.clone(),
    ))));
    instructions
}

/// The variant `shape` builds, around whatever the slot it is given holds.
fn building(shape: &Shape, around: Option<(u16, Descriptor)>) -> Vec<Instruction> {
    let mut instructions = vec![Instruction::New(shape.class.clone()), Instruction::Copy];
    if let Some((slot, of)) = around {
        instructions.push(Instruction::Load {
            slot,
            of: of.clone(),
        });
        instructions.extend(carried_as(shape, &of));
    }
    with(
        instructions,
        Instruction::Construct(MethodRef {
            class: shape.class.clone(),
            name: CONSTRUCTOR.to_owned(),
            descriptor: shape.constructor(),
        }),
    )
}

/// What the value on the stack becomes where the variant holds it as a reference and it is none.
fn carried_as(shape: &Shape, of: &Descriptor) -> Option<Instruction> {
    let [held] = shape.carries.as_slice() else {
        return None;
    };
    matches!(held.of, Some(Descriptor::Reference(_)))
        .then(|| as_a_reference(of))
        .flatten()
}

/// `instructions` with `last` after them, which is how one access is written in one expression.
fn with(mut instructions: Vec<Instruction>, last: Instruction) -> Vec<Instruction> {
    instructions.push(last);
    instructions
}

/// The class a `field` or a `static` names, which is every segment but the member's.
fn class_of(named: &JavaName) -> ClassName {
    let segments = named.segments();
    ClassName::new(&segments[..segments.len().saturating_sub(1)].join("/"))
}

/// The member a `field` or a `static` names, which is the last segment of its name.
fn member_of(named: &JavaName) -> String {
    named.segments().last().unwrap_or(&"").to_owned().to_owned()
}

/// The class `of` is, which every descriptor an `extern` writes a class into is one of.
fn class_named_by(of: &Descriptor) -> ClassName {
    let Descriptor::Reference(class) = of else {
        unreachable!("an extern reaching a class is held to a signature naming one")
    };
    class.clone()
}
