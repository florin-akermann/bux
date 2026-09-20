//! A name, with the module it is reached through where one is written.

use std::fmt;

use crate::{Name, Span};

/// `User`, or `demo.User`: a name, and the module it is reached inside of when one is written.
///
/// `docs/specs/modules.md` reaches a name inside a module through the module's name, and a type
/// and a pattern are reached that way exactly as a function is. The module is a name in scope
/// and the name after the dot is not, which is why the two are kept apart here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path {
    pub module: Option<Name>,
    pub name: Name,
}

impl Path {
    /// A name written on its own, which this module declares or the prelude supplies.
    #[must_use]
    pub const fn bare(name: Name) -> Self {
        Self { module: None, name }
    }

    /// The whole of what was written, as one name: the module, the dot, and the name after it.
    ///
    /// A diagnostic about the name says what the author wrote, and what the author wrote is
    /// `demo.User` where a module is written and `User` where none is.
    #[must_use]
    pub fn written(&self) -> Name {
        let Some(module) = &self.module else {
            return self.name.clone();
        };
        let start = module.span.start();
        Name {
            text: self.to_string(),
            span: Span::new(start, self.name.span.end() - start),
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(module) = &self.module else {
            return write!(f, "{}", self.name.text);
        };
        write!(f, "{}.{}", module.text, self.name.text)
    }
}
