//! What a call of a name inside a module runs, which is a static method of that module's class.
//!
//! A module loaded from a file is a class of its own, so a call reaching into one is the same
//! call as any other, written against the other class. A module the compiler supplies has no
//! class to reach, and is carried by a call of the JVM's own instead: `docs/specs/io.md` states
//! what each of those names means. Nothing of either is visible from Lumen, and a program that
//! writes `io.println` never learns what carries it.

use lumen_ast::{Expr, Name, Span};

use crate::class::{Class, Method, Reached};
use crate::code::{Body, FieldRef, Guard, Instruction, Label, MethodRef};
use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};
use crate::lower::Signature;
use crate::lower::body::Builder;
use crate::lower::shape::{CONSTRUCTOR, ERR, OK, Shapes};

/// The class holding the standard output a JVM starts with.
const SYSTEM: &str = "java/lang/System";

/// What that output is, and what writing a line is a method of.
const PRINT_STREAM: &str = "java/io/PrintStream";

const STRING: &str = "java/lang/String";

/// The file the path names, which is what a path is asked for.
const FILE: &str = "java/io/File";

/// What a file is reached by, and what reading one whole takes.
const PATH: &str = "java/nio/file/Path";

/// The class holding the read that takes a path and gives back the whole file as text.
const FILES: &str = "java/nio/file/Files";

/// What the read is asked for when it gives back a file rather than text.
const READ_WHOLE: &str = "readString";

/// Everything a JVM can throw, which is what the read is guarded against.
const THROWABLE: &str = "java/lang/Throwable";

/// What a throwable is asked to say about itself, which is what the `Err` holds.
const SAYS_OF_ITSELF: &str = "toString";

/// What a file is asked for before it is read, which is the path the read takes.
const AS_A_PATH: &str = "toPath";

/// The compiler's own class holding the read, which `files.read` is a call of.
const READER: &str = "lumen/Files";

/// What that class calls the read, which is what the Lumen name is called.
const READ: &str = "read";

/// The local the read's one parameter arrives in.
const PATH_HELD: u16 = 0;

/// The local the text of either answer is set aside in while the answer is built around it.
const TEXT_HELD: u16 = 1;

/// A name a call reaches through a module, and where the call writes it.
///
/// `used` is where the name is reached, which is what the types of the call are read off.
pub(crate) struct Through<'w> {
    pub(crate) module: &'w Name,
    pub(crate) name: &'w Name,
    pub(crate) used: Span,
}

impl Builder<'_> {
    /// A call of `module.name`, which inference has already settled the meaning of.
    pub(crate) fn inside_module(
        &mut self,
        reached: &Through<'_>,
        arguments: &[&Expr],
    ) -> Option<Descriptor> {
        if let Some(shape) = self
            .lowering
            .shapes
            .offered(&reached.module.text, &reached.name.text)
            .cloned()
        {
            return Some(self.builds(&shape, arguments));
        }
        match (reached.module.text.as_str(), reached.name.text.as_str()) {
            ("io", written @ ("print" | "println")) => self.written_out(written, arguments),
            ("files", "read") => Some(self.read_whole(arguments)),
            _ => self.in_another_module(reached, arguments),
        }
    }

    /// A call of a function of a module loaded from a file, which is a static method of it.
    ///
    /// What it takes and gives back is read off the use rather than off the other module's
    /// tree, which this module never holds. Inference has already met the two, so the use
    /// carries the very signature the other module wrote the method with.
    fn in_another_module(
        &mut self,
        reached: &Through<'_>,
        arguments: &[&Expr],
    ) -> Option<Descriptor> {
        let (named, signature) = self.reaches(reached);
        for (argument, wanted) in arguments.iter().zip(&signature.parameters) {
            self.handed(argument, wanted.clone());
        }
        self.emit(Instruction::InvokeStatic(MethodRef {
            class: ClassName::new(&reached.module.text),
            name: named,
            descriptor: signature.descriptor(),
        }));
        signature.result
    }

    /// The method this use reaches: what the other module calls it, and what it is written with.
    ///
    /// A function that declares no type parameter is written once, under its own name, and the
    /// use carries the very signature the other module wrote it with. A generic is written once
    /// per set of types it is used at, which `docs/specs/codegen.md` states, so a use of one
    /// reaches a method named for the set this use settled and written with the type that set
    /// gives the declaration, which is not the type the use has.
    ///
    /// A use inside a generic settles on that generic's own type parameters, and the body being
    /// written is one set of those, so the set asked for is what this set gives them.
    fn reaches(&mut self, reached: &Through<'_>) -> (String, Signature) {
        let Some(generic) = self.lowering.typed.generic_reached(reached.name.span) else {
            let signature = self.lowering.reached_through(reached.used);
            return (reached.name.text.clone(), signature);
        };
        let module = &reached.module.text;
        let asked_for = generic
            .settled()
            .iter()
            .map(|at| {
                self.lowering
                    .shapes
                    .as_written_by(module, &self.at().substituted(at))
            })
            .collect();
        let named = self.lowering.asking(module, &reached.name.text, asked_for);
        let written_as = self.at().substituted(generic.written_as());
        (named, self.lowering.written_as(&written_as))
    }

    /// Writing to the standard output, which stands on a class rather than on an instance.
    fn written_out(&mut self, named: &str, arguments: &[&Expr]) -> Option<Descriptor> {
        let text = Descriptor::reference(STRING);
        self.emit(Instruction::GetStatic(FieldRef {
            class: ClassName::new(SYSTEM),
            name: "out".to_owned(),
            of: Descriptor::reference(PRINT_STREAM),
        }));
        let [written] = arguments else {
            unreachable!("inference gave `io.{named}` the one argument it takes")
        };
        let held = self.expr(written);
        self.adapt(held, Some(text.clone()));
        self.emit(Instruction::InvokeVirtual(MethodRef {
            class: ClassName::new(PRINT_STREAM),
            name: named.to_owned(),
            descriptor: MethodDescriptor::new(vec![text], None),
        }));
        None
    }

    /// Reading a file whole, which is a call of the class that holds the guarded read.
    ///
    /// The read is a method of its own because a guarded span begins with nothing on the stack:
    /// what a throw discards is everything below it, and a call of `files.read` may be written
    /// anywhere an expression is, with any amount of a larger expression already worked out.
    fn read_whole(&mut self, arguments: &[&Expr]) -> Descriptor {
        let [written] = arguments else {
            unreachable!("inference gave `files.read` the one argument it takes")
        };
        let text = Descriptor::reference(STRING);
        let held = self.expr(written);
        self.adapt(held, Some(text.clone()));
        let read = reading(&self.lowering.shapes);
        let answer = read.descriptor.result.clone();
        self.emit(Instruction::InvokeStatic(read));
        answer.expect("a read gives back a `Result`, which a reference carries")
    }
}

/// Whether anything `class` does reaches the read, which is what decides if it is written.
pub(crate) fn reads_a_file(class: &Class) -> bool {
    class.methods.iter().any(|method| {
        method
            .body
            .instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::InvokeStatic(called) if called.class == ClassName::new(READER)))
    })
}

/// The class the read is a method of, which is written with a module that reads a file.
///
/// It is the compiler's own, the way the prelude classes are, and becomes Lumen source when a
/// module can be loaded from one.
pub(crate) fn files_class(shapes: &Shapes) -> Class {
    let mut class = Class::new(ClassName::new(READER));
    class.methods = vec![Method {
        name: READ.to_owned(),
        descriptor: reading(shapes).descriptor,
        reached: Reached::ThroughTheClass,
        body: read_body(shapes),
    }];
    class
}

/// The read as a call names it: one string in, and the `Result` both answers are, out.
fn reading(shapes: &Shapes) -> MethodRef {
    let answer = Descriptor::Reference(shapes.built(OK).base.clone());
    MethodRef {
        class: ClassName::new(READER),
        name: READ.to_owned(),
        descriptor: MethodDescriptor::new(vec![Descriptor::reference(STRING)], Some(answer)),
    }
}

/// What the read does: the file is read under a guard, and each answer is built and given back.
fn read_body(shapes: &Shapes) -> Body {
    let text = Descriptor::reference(STRING);
    let guard = Guard {
        from: Label(0),
        to: Label(1),
        handler: Label(2),
        catching: ClassName::new(THROWABLE),
    };
    let mut instructions = vec![Instruction::Label(guard.from)];
    instructions.extend(opened(PATH_HELD, &text));
    instructions.push(Instruction::Label(guard.to));
    instructions.extend(answered(shapes, OK, &text));
    instructions.push(Instruction::Label(guard.handler));
    instructions.push(Instruction::InvokeVirtual(MethodRef {
        class: ClassName::new(THROWABLE),
        name: SAYS_OF_ITSELF.to_owned(),
        descriptor: MethodDescriptor::new(Vec::new(), Some(text.clone())),
    }));
    instructions.extend(answered(shapes, ERR, &text));
    Body {
        instructions,
        locals: 1,
        guards: vec![guard],
    }
}

/// The file the path in `slot` names, read whole, which is all the guard covers.
fn opened(slot: u16, text: &Descriptor) -> Vec<Instruction> {
    let path = Descriptor::reference(PATH);
    vec![
        Instruction::New(ClassName::new(FILE)),
        Instruction::Copy,
        Instruction::Load {
            slot,
            of: text.clone(),
        },
        Instruction::Construct(MethodRef {
            class: ClassName::new(FILE),
            name: CONSTRUCTOR.to_owned(),
            descriptor: MethodDescriptor::new(vec![text.clone()], None),
        }),
        Instruction::InvokeVirtual(MethodRef {
            class: ClassName::new(FILE),
            name: AS_A_PATH.to_owned(),
            descriptor: MethodDescriptor::new(Vec::new(), Some(path.clone())),
        }),
        Instruction::InvokeStatic(MethodRef {
            class: ClassName::new(FILES),
            name: READ_WHOLE.to_owned(),
            descriptor: MethodDescriptor::new(vec![path], Some(text.clone())),
        }),
    ]
}

/// The variant `named` built around the text on the stack, and given back.
fn answered(shapes: &Shapes, named: &str, text: &Descriptor) -> Vec<Instruction> {
    let shape = shapes.built(named);
    vec![
        Instruction::Store {
            slot: TEXT_HELD,
            of: text.clone(),
        },
        Instruction::New(shape.class.clone()),
        Instruction::Copy,
        Instruction::Load {
            slot: TEXT_HELD,
            of: text.clone(),
        },
        Instruction::Construct(MethodRef {
            class: shape.class.clone(),
            name: CONSTRUCTOR.to_owned(),
            descriptor: shape.constructor(),
        }),
        Instruction::Return(Some(Descriptor::Reference(shape.base.clone()))),
    ]
}
