//! Where an import belongs, which is part of canonical form and not of the text a printer writes.
//!
//! `docs/design.md` section 13 puts imports first, sorted by the module they name. The rule is
//! checked and never rewritten: `lumen fmt` repairs whitespace, which is nobody's decision, and
//! says where an import belongs rather than moving it there.

use lumen_ast::{Import, Item, Program, Span};

/// The first import of `program` that is out of place, when one is.
pub(crate) fn out_of_order(program: &Program) -> Option<OutOfOrder> {
    let mut declared = false;
    let mut last: Option<&Import> = None;
    for item in &program.items {
        let Item::Import(import) = item else {
            declared = true;
            continue;
        };
        if declared {
            return Some(OutOfOrder::afterwards(import));
        }
        if let Some(before) = last.filter(|before| before.module.text > import.module.text) {
            return Some(OutOfOrder::unsorted(import, before));
        }
        last = Some(import);
    }
    None
}

/// An import written somewhere canonical form does not put it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutOfOrder {
    /// The import that is out of place, which is the one the reader is pointed at.
    module: String,
    /// What it is out of place with respect to, which is what the reader has to look for.
    against: Against,
    span: Span,
}

/// What an import is out of place against.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Against {
    /// Something that is not an import is written above it.
    Declaration,
    /// The import above it names a module that sorts after it.
    Import(String),
}

impl OutOfOrder {
    /// The import the reader is pointed at.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// What to do about it, which is where the import belongs.
    #[must_use]
    pub fn help(&self) -> String {
        match &self.against {
            Against::Declaration => "imports come first, before every declaration".to_owned(),
            Against::Import(before) => {
                format!("`{}` sorts before `{before}`", self.module)
            }
        }
    }

    fn afterwards(import: &Import) -> Self {
        Self {
            module: import.module.text.clone(),
            against: Against::Declaration,
            span: import.span,
        }
    }

    fn unsorted(import: &Import, before: &Import) -> Self {
        Self {
            module: import.module.text.clone(),
            against: Against::Import(before.module.text.clone()),
            span: import.span,
        }
    }
}

impl std::fmt::Display for OutOfOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.against {
            Against::Declaration => {
                write!(f, "`import {}` is written after a declaration", self.module)
            }
            Against::Import(before) => write!(
                f,
                "`import {}` is written after `import {before}`",
                self.module
            ),
        }
    }
}
