//! The constructors of each type, which is what a `match` on it has to cover.

use std::collections::HashMap;

use lumen_ast::{Item, Program, TypeDeclaration, TypeDefinition, VariantPayload};

/// One constructor, and how many values it carries.
///
/// What it carries is counted rather than typed: exhaustiveness asks how many patterns sit under
/// a constructor, never what they match, because inference has settled that already.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Signature {
    pub(crate) name: String,
    pub(crate) carries: usize,
}

/// Every constructor a module can write, grouped by the type that declares it.
///
/// A constructor is looked up by name alone, which name resolution makes unambiguous: a module
/// declares each name once, so no two types share a constructor name.
#[derive(Clone, Debug, Default)]
pub(crate) struct Space {
    declared: Vec<Vec<Signature>>,
    of_constructor: HashMap<String, usize>,
}

impl Space {
    /// The constructors of `program`, on top of the ones every module has.
    pub(crate) fn of(program: &Program) -> Self {
        let mut space = Self::default();
        space.declare(vec![signature("false", 0), signature("true", 0)]);
        space.declare(vec![signature("Some", 1), signature("None", 0)]);
        space.declare(vec![signature("Ok", 1), signature("Err", 1)]);
        for item in &program.items {
            if let Item::Type(declaration) = item {
                space.declare(constructors(declaration));
            }
        }
        space
    }

    /// How many values `constructor` carries, which is none when nothing declares it.
    pub(crate) fn carried_by(&self, constructor: &str) -> usize {
        self.siblings(constructor)
            .and_then(|siblings| siblings.iter().find(|one| one.name == constructor))
            .map_or(0, |found| found.carries)
    }

    /// Where `constructor` sits among the constructors its type declares.
    pub(crate) fn declared_at(&self, constructor: &str) -> Option<usize> {
        self.siblings(constructor)?
            .iter()
            .position(|one| one.name == constructor)
    }

    /// The constructors of the type `constructor` belongs to, when a type declares them.
    ///
    /// A literal has none: `Int` and `String` have more values than a declaration writes down.
    pub(crate) fn siblings(&self, constructor: &str) -> Option<&[Signature]> {
        let declared = *self.of_constructor.get(constructor)?;
        Some(&self.declared[declared])
    }

    fn declare(&mut self, signatures: Vec<Signature>) {
        let declared = self.declared.len();
        for signature in &signatures {
            self.of_constructor.insert(signature.name.clone(), declared);
        }
        self.declared.push(signatures);
    }
}

/// What a type declaration declares: one constructor for a record, and one for each variant.
fn constructors(declaration: &TypeDeclaration) -> Vec<Signature> {
    match &declaration.definition {
        TypeDefinition::Record(fields) => {
            vec![signature(&declaration.name.text, fields.len())]
        }
        TypeDefinition::Variants(variants) => variants
            .iter()
            .map(|variant| signature(&variant.name.text, carried(&variant.payload)))
            .collect(),
    }
}

const fn carried(payload: &VariantPayload) -> usize {
    match payload {
        VariantPayload::None => 0,
        VariantPayload::Tuple(types) => types.len(),
        VariantPayload::Record(fields) => fields.len(),
    }
}

fn signature(name: &str, carries: usize) -> Signature {
    Signature {
        name: name.to_owned(),
        carries,
    }
}
