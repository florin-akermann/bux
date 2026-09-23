//! Writing class files: a lowered program as the bytes a JVM loads.
//!
//! The phase consumes every lowered module of a program and yields one [`ClassFile`] per class
//! they describe. `docs/specs/codegen.md` is the specification, and the class-file version is the
//! current JDK's. Nothing here knows Lumen: it knows classes, descriptors, and opcodes.

mod bytes;
mod class;
mod code;
mod emit;
mod frame;
mod opcode;
mod pool;

use lumen_ir::Lowered;

/// One class, and where a build writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassFile {
    /// The path the file takes, relative to where the build writes, such as `demo/User.class`.
    pub path: String,
    pub bytes: Vec<u8>,
}

/// Writes every class of every module of `program`, in the order the modules name them.
///
/// The modules are written together because a frame names the class two branches share, and a
/// variant one module gives back may extend a base another module declares.
#[must_use]
pub fn write(program: &[Lowered]) -> Vec<ClassFile> {
    let classes = || program.iter().flat_map(|module| &module.classes);
    let hierarchy =
        frame::Hierarchy::of(classes().map(|class| (class.name.clone(), class.extends.clone())));
    classes()
        .map(|class| ClassFile {
            path: class.name.path(),
            bytes: class::write(class, &hierarchy),
        })
        .collect()
}
