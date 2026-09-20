//! What the verifier is told holds at each place a jump lands.

use std::collections::HashMap;

use lumen_ir::{ClassName, Descriptor};

use crate::bytes::Bytes;
use crate::pool::Pool;

/// What one value is, as the verifier counts them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Held {
    Integer,
    Long,
    Object(ClassName),
    /// An instance made but not yet built, named by where it was made.
    ///
    /// The verifier refuses to do anything with one but hand it to a constructor, and the JVM
    /// needs to be told that is what it is holding wherever a jump lands.
    Uninitialised(u16),
}

/// One local slot: what it holds, or the second half of a whole number, or nothing yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Slot {
    Holding(Held),
    /// The slot after a whole number, which a class file leaves out of a frame.
    Second,
    Nothing,
}

/// What holds where control is, which is what a frame says.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct Frame {
    pub(crate) locals: Vec<Slot>,
    pub(crate) stack: Vec<Held>,
}

impl Frame {
    /// What a method starts with: what it is reached through, then its parameters.
    pub(crate) fn entering(
        parameters: &[Descriptor],
        receiver: Option<&ClassName>,
        slots: u16,
    ) -> Self {
        let mut frame = Self {
            locals: vec![Slot::Nothing; slots as usize],
            stack: Vec::new(),
        };
        let mut at = 0;
        if let Some(receiver) = receiver {
            frame.locals[0] = Slot::Holding(Held::Object(receiver.clone()));
            at = 1;
        }
        for parameter in parameters {
            frame.hold(at, parameter);
            at += parameter.width();
        }
        frame
    }

    pub(crate) fn hold(&mut self, slot: u16, descriptor: &Descriptor) {
        let at = slot as usize;
        self.locals[at] = Slot::Holding(Held::of(descriptor));
        if descriptor.is_wide() {
            self.locals[at + 1] = Slot::Second;
        }
    }

    /// Says that every copy of the instance made at `at` now holds one that is built.
    pub(crate) fn initialised(&mut self, at: u16, class: &ClassName) {
        let made = Held::Uninitialised(at);
        let built = Held::Object(class.clone());
        let in_locals = self.locals.iter_mut().filter_map(|slot| match slot {
            Slot::Holding(held) => Some(held),
            Slot::Second | Slot::Nothing => None,
        });
        for held in self.stack.iter_mut().chain(in_locals) {
            if *held == made {
                *held = built.clone();
            }
        }
    }

    /// The stack words this frame holds, which is what `max_stack` is the largest of.
    pub(crate) fn depth(&self) -> usize {
        self.stack.iter().map(Held::width).sum()
    }

    /// What holds on both paths into one place, which is what the verifier is told.
    ///
    /// A slot that holds one thing on one path and another on the other holds nothing, and a
    /// value that is one class on one path and another on the other is the class they share.
    pub(crate) fn merged_with(&self, other: &Self, hierarchy: &Hierarchy) -> Self {
        let locals = self
            .locals
            .iter()
            .zip(&other.locals)
            .map(|(mine, theirs)| {
                if mine == theirs {
                    mine.clone()
                } else {
                    Slot::Nothing
                }
            })
            .collect();
        let stack = self
            .stack
            .iter()
            .zip(&other.stack)
            .map(|(mine, theirs)| hierarchy.shared_by(mine, theirs))
            .collect();
        Self { locals, stack }
    }

    /// Writes this as a full frame, which is the one form that says everything.
    pub(crate) fn write(&self, delta: u16, pool: &mut Pool, bytes: &mut Bytes) {
        let named = self.named();
        bytes.u1(255);
        bytes.u2(delta);
        bytes.u2(u16::try_from(named.len()).unwrap_or(u16::MAX));
        for held in named {
            match held {
                Some(held) => held.write(pool, bytes),
                None => bytes.u1(0),
            }
        }
        bytes.u2(u16::try_from(self.stack.len()).unwrap_or(u16::MAX));
        for held in &self.stack {
            held.write(pool, bytes);
        }
    }

    /// The locals as a frame writes them.
    ///
    /// The second half of a whole number is left out, a slot holding nothing yet is written as
    /// the verifier's `Top`, and the slots after the last one holding anything are left off.
    fn named(&self) -> Vec<Option<&Held>> {
        let last = self
            .locals
            .iter()
            .rposition(|slot| matches!(slot, Slot::Holding(_)))
            .map_or(0, |at| at + 1);
        self.locals[..last]
            .iter()
            .filter_map(|slot| match slot {
                Slot::Holding(held) => Some(Some(held)),
                Slot::Nothing => Some(None),
                Slot::Second => None,
            })
            .collect()
    }
}

impl Held {
    /// What a value of this descriptor is; a truth value is a small whole number to the verifier.
    pub(crate) fn of(descriptor: &Descriptor) -> Self {
        match descriptor {
            Descriptor::Long => Self::Long,
            Descriptor::Boolean | Descriptor::Integer => Self::Integer,
            Descriptor::Reference(class) => Self::Object(class.clone()),
            // An array names itself the way a descriptor writes it, which is what the class a
            // frame points at is called when the thing it holds is an array.
            Descriptor::Array(held) => Self::Object(ClassName::new(&format!("[{held}"))),
        }
    }

    /// The stack words this takes, which is two for a whole number.
    pub(crate) const fn width(&self) -> usize {
        if self.is_wide() { 2 } else { 1 }
    }

    /// Whether this takes two stack words rather than one.
    pub(crate) const fn is_wide(&self) -> bool {
        matches!(self, Self::Long)
    }

    fn write(&self, pool: &mut Pool, bytes: &mut Bytes) {
        match self {
            Self::Integer => bytes.u1(1),
            Self::Long => bytes.u1(4),
            Self::Object(class) => {
                let named = pool.class(class);
                bytes.u1(7);
                bytes.u2(named);
            }
            Self::Uninitialised(at) => {
                bytes.u1(8);
                bytes.u2(*at);
            }
        }
    }
}

/// What extends what, which is how two classes are found to share one.
#[derive(Debug, Default)]
pub(crate) struct Hierarchy {
    extends: HashMap<ClassName, ClassName>,
}

impl Hierarchy {
    pub(crate) fn of(pairs: impl Iterator<Item = (ClassName, ClassName)>) -> Self {
        Self {
            extends: pairs.collect(),
        }
    }

    /// Whether a value of `of` is one a JVM may fold into the class holding it.
    ///
    /// Only a class this build writes is a value class to begin with, so a field carried by one
    /// the JVM ships is never folded. Nor is the base of a sum type, which its own variants
    /// extend: a field typed as one holds whichever variant it was given, so it stays a
    /// reference however early the base is loaded, and asking for it early buys nothing.
    pub(crate) fn is_foldable(&self, of: &Descriptor) -> bool {
        let Descriptor::Reference(class) = of else {
            return false;
        };
        self.extends.contains_key(class) && !self.extends.values().any(|base| base == class)
    }

    /// The class both are, which is one of them, the one they extend, or `java.lang.Object`.
    fn shared_by(&self, mine: &Held, theirs: &Held) -> Held {
        let (Held::Object(mine), Held::Object(theirs)) = (mine, theirs) else {
            return mine.clone();
        };
        if mine == theirs {
            return Held::Object(mine.clone());
        }
        let object = ClassName::new("java/lang/Object");
        let above_mine = self.extends.get(mine).unwrap_or(&object);
        let above_theirs = self.extends.get(theirs).unwrap_or(&object);
        if above_mine == theirs {
            return Held::Object(theirs.clone());
        }
        if above_theirs == mine {
            return Held::Object(mine.clone());
        }
        if above_mine == above_theirs {
            return Held::Object(above_mine.clone());
        }
        Held::Object(object)
    }
}
