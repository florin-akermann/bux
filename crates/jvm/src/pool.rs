//! The constant pool: everything a class file names, each named once.

use std::collections::HashMap;

use lumen_ir::{ClassName, FieldRef, MethodRef};

use crate::bytes::Bytes;

/// One entry, as the pool holds it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Entry {
    Utf8(String),
    Integer(i32),
    Long(i64),
    Text(u16),
    Class(u16),
    NameAndType { name: u16, descriptor: u16 },
    Field { class: u16, member: u16 },
    Method { class: u16, member: u16 },
}

/// Everything one class file names.
///
/// An entry is added the first time it is asked for and shared afterwards, so the pool is fixed
/// by the order the writer walks the class. That is what makes the output reproducible.
#[derive(Debug, Default)]
pub(crate) struct Pool {
    entries: Vec<Entry>,
    known: HashMap<Entry, u16>,
    /// The index the next entry takes, counting from one as a class file does.
    next: u16,
}

impl Pool {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            known: HashMap::new(),
            next: 1,
        }
    }

    pub(crate) fn text(&mut self, value: &str) -> u16 {
        let held = self.utf8(value);
        self.add(Entry::Text(held))
    }

    pub(crate) fn field(&mut self, field: &FieldRef) -> u16 {
        let class = self.class(&field.class);
        let member = self.name_and_type(&field.name, &field.of.to_string());
        self.add(Entry::Field { class, member })
    }

    pub(crate) fn method(&mut self, method: &MethodRef) -> u16 {
        let class = self.class(&method.class);
        let member = self.name_and_type(&method.name, &method.descriptor.to_string());
        self.add(Entry::Method { class, member })
    }

    fn name_and_type(&mut self, name: &str, descriptor: &str) -> u16 {
        let name = self.utf8(name);
        let descriptor = self.utf8(descriptor);
        self.add(Entry::NameAndType { name, descriptor })
    }

    pub(crate) fn class(&mut self, name: &ClassName) -> u16 {
        let written = self.utf8(name.written());
        self.add(Entry::Class(written))
    }

    pub(crate) fn utf8(&mut self, text: &str) -> u16 {
        self.add(Entry::Utf8(text.to_owned()))
    }

    pub(crate) fn integer(&mut self, value: i32) -> u16 {
        self.add(Entry::Integer(value))
    }

    pub(crate) fn long(&mut self, value: i64) -> u16 {
        self.add(Entry::Long(value))
    }

    fn add(&mut self, entry: Entry) -> u16 {
        if let Some(known) = self.known.get(&entry) {
            return *known;
        }
        let at = self.next;
        self.known.insert(entry.clone(), at);
        // A long takes two indices, the second of which a class file leaves unusable.
        self.next += if matches!(entry, Entry::Long(_)) {
            2
        } else {
            1
        };
        self.entries.push(entry);
        at
    }

    /// What a class file writes for `constant_pool_count`, which is one past the last index.
    pub(crate) const fn count(&self) -> u16 {
        self.next
    }

    pub(crate) fn write(&self, bytes: &mut Bytes) {
        for entry in &self.entries {
            write_entry(entry, bytes);
        }
    }
}

fn write_entry(entry: &Entry, bytes: &mut Bytes) {
    match entry {
        Entry::Utf8(text) => {
            bytes.u1(1);
            let written = modified_utf8(text);
            bytes.u2(u16::try_from(written.len()).unwrap_or(u16::MAX));
            bytes.all(&written);
        }
        Entry::Integer(value) => {
            bytes.u1(3);
            bytes.u4(value.cast_unsigned());
        }
        Entry::Long(value) => {
            bytes.u1(5);
            bytes.all(&value.to_be_bytes());
        }
        Entry::Text(held) => {
            bytes.u1(8);
            bytes.u2(*held);
        }
        Entry::Class(written) => {
            bytes.u1(7);
            bytes.u2(*written);
        }
        Entry::NameAndType { name, descriptor } => {
            bytes.u1(12);
            bytes.u2(*name);
            bytes.u2(*descriptor);
        }
        Entry::Field { class, member } => {
            bytes.u1(9);
            bytes.u2(*class);
            bytes.u2(*member);
        }
        Entry::Method { class, member } => {
            bytes.u1(10);
            bytes.u2(*class);
            bytes.u2(*member);
        }
    }
}

/// `text` in the encoding a class file calls UTF-8, which is not quite the one Rust means.
///
/// A class file writes a zero byte as two bytes so that no string holds one, and writes a
/// character above the basic plane as the two halves it is made of rather than as four bytes.
fn modified_utf8(text: &str) -> Vec<u8> {
    let mut written = Vec::with_capacity(text.len());
    for held in text.chars() {
        match held {
            '\0' => written.extend_from_slice(&[0xC0, 0x80]),
            held if (held as u32) < 0x1_0000 => {
                let mut encoded = [0; 4];
                written.extend_from_slice(held.encode_utf8(&mut encoded).as_bytes());
            }
            held => {
                let mut halves = [0; 2];
                for half in held.encode_utf16(&mut halves) {
                    written.extend_from_slice(&three_bytes(*half));
                }
            }
        }
    }
    written
}

/// One half of a character above the basic plane, which is always three bytes.
const fn three_bytes(half: u16) -> [u8; 3] {
    [
        0xE0 | ((half >> 12) as u8),
        0x80 | (((half >> 6) & 0x3F) as u8),
        0x80 | ((half & 0x3F) as u8),
    ]
}
