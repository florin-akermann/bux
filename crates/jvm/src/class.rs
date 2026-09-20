//! One class, as the bytes a JVM loads.

use lumen_ir::{Class, Extending, Method, Reached};

use crate::bytes::Bytes;
use crate::code::{Assembled, Caught, Context, assemble};
use crate::frame::Hierarchy;
use crate::pool::Pool;

/// The minor and major class-file version: JDK 28, marked preview, which value classes are there.
///
/// A preview class file carries minor version 65535, and a JVM loads one only when started with
/// `--enable-preview`, which `lumen run` passes; `docs/implementation.md` section 1 says why.
const VERSION: (u16, u16) = (65535, 72);

/// `ACC_PUBLIC` alone: the identity bit, `0x0020`, is left clear, which makes a value class.
const CLASS_ACCESS: u16 = 0x0001;
const FINAL: u16 = 0x0010;
const ABSTRACT: u16 = 0x0400;
/// `ACC_STRICT_INIT`: the field is written before the constructor hands itself up, which every
/// field of a value class is.
const STRICT_INIT: u16 = 0x0800;
const FIELD_ACCESS: u16 = 0x0001 | FINAL | STRICT_INIT;
const METHOD_ACCESS: u16 = 0x0001;
const STATIC: u16 = 0x0008;

/// Writes `class` as the bytes a JVM loads.
pub(crate) fn write(class: &Class, hierarchy: &Hierarchy) -> Vec<u8> {
    let mut pool = Pool::new();
    let mut context = Context {
        pool: &mut pool,
        hierarchy,
    };
    let named = context.pool.class(&class.name);
    let extends = context.pool.class(&class.extends);
    let fields = written_fields(class, &mut context);
    let methods = written_methods(class, &mut context);
    let wanted = written_loadable(class, &mut context);
    let mut bytes = Bytes::default();
    bytes.u4(0xCAFE_BABE);
    bytes.u2(VERSION.0);
    bytes.u2(VERSION.1);
    bytes.u2(pool.count());
    pool.write(&mut bytes);
    bytes.u2(CLASS_ACCESS | extending(class.extending));
    bytes.u2(named);
    bytes.u2(extends);
    bytes.u2(0);
    bytes.all(&fields);
    bytes.all(&methods);
    bytes.u2(u16::from(wanted.is_some()));
    bytes.all(wanted.as_deref().unwrap_or_default());
    bytes.taken()
}

/// The `LoadableDescriptors` attribute, where this class waits on another to lay itself out.
///
/// A JVM decides where each field of a class sits while it loads that class, and it can fold a
/// value into one only if it already knows what that value holds. JEP 401 lets a class file say
/// which descriptors it wants loaded first, and `docs/specs/codegen.md` states which ones a class
/// names. A class that waits on nothing carries no attribute rather than an empty one.
fn written_loadable(class: &Class, context: &mut Context<'_>) -> Option<Vec<u8>> {
    let mut wanted: Vec<u16> = Vec::new();
    for field in &class.fields {
        if !context.hierarchy.is_foldable(&field.of) {
            continue;
        }
        let descriptor = context.pool.utf8(&field.of.to_string());
        if !wanted.contains(&descriptor) {
            wanted.push(descriptor);
        }
    }
    if wanted.is_empty() {
        return None;
    }
    let named = context.pool.utf8("LoadableDescriptors");
    let mut descriptors = Bytes::default();
    descriptors.u2(u16::try_from(wanted.len()).unwrap_or_default());
    for descriptor in wanted {
        descriptors.u2(descriptor);
    }
    let written = descriptors.taken();
    let mut bytes = Bytes::default();
    bytes.u2(named);
    bytes.u4(u32::try_from(written.len()).unwrap_or_default());
    bytes.all(&written);
    Some(bytes.taken())
}

const fn extending(extending: Extending) -> u16 {
    match extending {
        Extending::Never => FINAL,
        Extending::ByItsVariants => ABSTRACT,
    }
}

fn written_fields(class: &Class, context: &mut Context<'_>) -> Vec<u8> {
    let mut bytes = Bytes::default();
    bytes.u2(u16::try_from(class.fields.len()).unwrap_or_default());
    for field in &class.fields {
        let name = context.pool.utf8(&field.name);
        let descriptor = context.pool.utf8(&field.of.to_string());
        bytes.u2(FIELD_ACCESS);
        bytes.u2(name);
        bytes.u2(descriptor);
        bytes.u2(0);
    }
    bytes.taken()
}

fn written_methods(class: &Class, context: &mut Context<'_>) -> Vec<u8> {
    let mut bytes = Bytes::default();
    bytes.u2(u16::try_from(class.methods.len()).unwrap_or_default());
    for method in &class.methods {
        written_method(method, class, context, &mut bytes);
    }
    bytes.taken()
}

fn written_method(method: &Method, class: &Class, context: &mut Context<'_>, bytes: &mut Bytes) {
    let name = context.pool.utf8(&method.name);
    let descriptor = context.pool.utf8(&method.descriptor.to_string());
    let code = written_code(method, class, context);
    bytes.u2(METHOD_ACCESS | reached(method.reached));
    bytes.u2(name);
    bytes.u2(descriptor);
    bytes.u2(1);
    bytes.all(&code);
}

const fn reached(reached: Reached) -> u16 {
    match reached {
        Reached::ThroughTheClass => STATIC,
        Reached::ThroughAnInstance => 0,
    }
}

/// The `Code` attribute of one method, with the stack map its branches need.
fn written_code(method: &Method, class: &Class, context: &mut Context<'_>) -> Vec<u8> {
    let receiver = match method.reached {
        Reached::ThroughTheClass => None,
        Reached::ThroughAnInstance => Some(class.name.clone()),
    };
    let assembled = assemble(&method.body, &method.descriptor, receiver.as_ref(), context);
    let caught = written_handlers(&assembled.handlers, context.pool);
    let map = written_map(&assembled, context.pool);
    let named = context.pool.utf8("Code");
    let mut bytes = Bytes::default();
    bytes.u2(assembled.max_stack);
    bytes.u2(assembled.max_locals);
    bytes.u4(u32::try_from(assembled.code.len()).unwrap_or_default());
    bytes.all(&assembled.code);
    bytes.all(&caught);
    bytes.u2(u16::from(!map.is_empty()));
    bytes.all(&map);
    let body = bytes.taken();
    let mut attribute = Bytes::default();
    attribute.u2(named);
    attribute.u4(u32::try_from(body.len()).unwrap_or_default());
    attribute.all(&body);
    attribute.taken()
}

/// The exception table, which names every span whose failure is caught and where it lands.
fn written_handlers(handlers: &[Caught], pool: &mut Pool) -> Vec<u8> {
    let mut bytes = Bytes::default();
    bytes.u2(u16::try_from(handlers.len()).unwrap_or_default());
    for caught in handlers {
        let catching = pool.class(&caught.catching);
        bytes.u2(caught.from);
        bytes.u2(caught.to);
        bytes.u2(caught.handler);
        bytes.u2(catching);
    }
    bytes.taken()
}

/// The `StackMapTable` attribute, which is left out when nothing branches.
fn written_map(assembled: &Assembled, pool: &mut Pool) -> Vec<u8> {
    if assembled.frames.is_empty() {
        return Vec::new();
    }
    let mut entries = Bytes::default();
    entries.u2(u16::try_from(assembled.frames.len()).unwrap_or_default());
    let mut previous: Option<u16> = None;
    for (at, frame) in &assembled.frames {
        let delta = previous.map_or(*at, |last| at - last - 1);
        frame.write(delta, pool, &mut entries);
        previous = Some(*at);
    }
    let written = entries.taken();
    let named = pool.utf8("StackMapTable");
    let mut bytes = Bytes::default();
    bytes.u2(named);
    bytes.u4(u32::try_from(written.len()).unwrap_or_default());
    bytes.all(&written);
    bytes.taken()
}
