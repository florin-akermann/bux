//! Writing class files: a lowered module as the bytes a JVM loads.
//!
//! The phase consumes the lowered program and yields one [`ClassFile`] per class it describes.
//! `docs/specs/codegen.md` is the specification, and the class-file version is the current JDK's.
//! Nothing here knows Lumen: it knows classes, descriptors, and opcodes.

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

/// Writes every class of `lowered`, in the order it names them.
#[must_use]
pub fn write(lowered: &Lowered) -> Vec<ClassFile> {
    let hierarchy = frame::Hierarchy::of(
        lowered
            .classes
            .iter()
            .map(|class| (class.name.clone(), class.extends.clone())),
    );
    lowered
        .classes
        .iter()
        .map(|class| ClassFile {
            path: class.name.path(),
            bytes: class::write(class, &hierarchy),
        })
        .collect()
}
