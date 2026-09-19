//! What each type a module declares is laid out as.
//!
//! Every constructor the source can write builds one class, and every type it can write is
//! carried by one descriptor. `docs/specs/codegen.md` states the layout; this is where a name
//! written in Lumen becomes a name the JVM knows.

use std::collections::HashMap;

use lumen_ast::{Item, RecordField, TypeDeclaration, TypeDefinition, TypeRef, TypeRefKind};
use lumen_ast::{Variant, VariantPayload};
use lumen_resolver::{DefinitionKind, Namespace, ResolvedProgram};
use lumen_types::Type;

use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};

/// The types the prelude supplies, each with its variants and what each variant carries.
///
/// The order is the order a tag counts in, and it is the order exhaustiveness lists them in.
const PRELUDE: [(&str, [(&str, usize); 2]); 2] = [
    ("Option", [("Some", 1), ("None", 0)]),
    ("Result", [("Ok", 1), ("Err", 1)]),
];

/// The package the prelude types are written in, which every module may reach.
const PRELUDE_PACKAGE: &str = "lumen";

/// The field an algebraic data type carries, which says which of its variants a value is.
pub(crate) const TAG: &str = "tag";

/// What the JVM calls a constructor.
pub(crate) const CONSTRUCTOR: &str = "<init>";

/// The variant of `Option` a division with an answer builds.
pub(crate) const SOME: &str = "Some";

/// The variant of `Option` a division by zero builds.
pub(crate) const NONE: &str = "None";

/// The variant of `Result` a `?` reads the value out of.
pub(crate) const OK: &str = "Ok";

/// The variant of `Result` a `?` gives back as it was handed.
pub(crate) const ERR: &str = "Err";

/// One value a constructor holds, which is carried by nothing when its type is `()`.
#[derive(Clone, Debug)]
pub(crate) struct Carried {
    pub(crate) name: String,
    pub(crate) of: Option<Descriptor>,
}

/// The class one constructor builds, and what that class holds.
#[derive(Clone, Debug)]
pub(crate) struct Shape {
    pub(crate) class: ClassName,
    /// The class a value of the type is held as: an ADT's base, or the record's own class.
    pub(crate) base: ClassName,
    /// The variant's position in its declaration; a record is its type, so it has no tag.
    pub(crate) tag: Option<i32>,
    pub(crate) carries: Vec<Carried>,
}

impl Shape {
    /// The descriptor of the constructor, which takes every value the class holds.
    pub(crate) fn constructor(&self) -> MethodDescriptor {
        let taken = self
            .carries
            .iter()
            .filter_map(|held| held.of.clone())
            .collect();
        MethodDescriptor::new(taken, None)
    }
}

/// One type declaration, as the classes it becomes.
#[derive(Clone, Debug)]
pub(crate) enum Declared {
    /// A record, which is one class built with one constructor of the same name.
    Record(String),
    /// An algebraic data type: a base nothing but its own variants extend.
    Variants {
        base: ClassName,
        constructors: Vec<String>,
    },
}

/// Every class a module's declarations become, and what each constructor builds.
#[derive(Debug)]
pub(crate) struct Shapes {
    module: ClassName,
    declarations: Vec<Declared>,
    built_by: HashMap<String, Shape>,
    /// The constructor of each record type, which is what a field is reached through.
    records: HashMap<String, String>,
}

impl Shapes {
    /// The shapes `resolved` declares, in a module of this name, with the prelude beside them.
    pub(crate) fn of(resolved: &ResolvedProgram, module: &str) -> Self {
        let mut shapes = Self {
            module: ClassName::new(module),
            declarations: Vec::new(),
            built_by: HashMap::new(),
            records: HashMap::new(),
        };
        for item in &resolved.program().items {
            if let Item::Type(declaration) = item {
                shapes.declare(resolved, declaration);
            }
        }
        shapes.declare_prelude();
        shapes
    }

    /// The class the module's own functions are static methods of.
    pub(crate) const fn module(&self) -> &ClassName {
        &self.module
    }

    /// Every type declaration, in the order the module writes them, the prelude last.
    pub(crate) fn declarations(&self) -> &[Declared] {
        &self.declarations
    }

    /// The one constructor of the record type called `named`, which is what carries its fields.
    pub(crate) fn record(&self, named: &str) -> &Shape {
        let constructor = self
            .records
            .get(named)
            .expect("a field is reached only through a record type");
        self.built(constructor)
    }

    /// What a value of `of` is carried by, which is nothing at all when it is `()`.
    pub(crate) fn carried(&self, of: &Type) -> Option<Descriptor> {
        match of {
            Type::Unit => None,
            Type::Named { name, .. } => Some(self.named(name)),
            Type::Var(_) | Type::Parameter(_) | Type::Function { .. } | Type::Module(_) => {
                Some(object())
            }
        }
    }

    fn declare(&mut self, resolved: &ResolvedProgram, declaration: &TypeDeclaration) {
        let base = self.declared(&declaration.name.text);
        let TypeDefinition::Variants(variants) = &declaration.definition else {
            let TypeDefinition::Record(fields) = &declaration.definition else {
                unreachable!("a type declaration is a record or variants")
            };
            let carries = self.fields(resolved, fields);
            self.records
                .insert(declaration.name.text.clone(), declaration.name.text.clone());
            self.declarations
                .push(Declared::Record(declaration.name.text.clone()));
            self.built_by.insert(
                declaration.name.text.clone(),
                Shape {
                    class: base.clone(),
                    base,
                    tag: None,
                    carries,
                },
            );
            return;
        };
        let mut constructors = Vec::new();
        for (position, variant) in variants.iter().enumerate() {
            let carries = self.payload(resolved, variant);
            let class = variant_class(&base, &variant.name.text);
            let tag = i32::try_from(position).unwrap_or_default();
            constructors.push(variant.name.text.clone());
            self.built_by.insert(
                variant.name.text.clone(),
                Shape {
                    class,
                    base: base.clone(),
                    tag: Some(tag),
                    carries,
                },
            );
        }
        self.declarations
            .push(Declared::Variants { base, constructors });
    }

    /// `Option` and `Result`, which are in scope in every module without being declared.
    fn declare_prelude(&mut self) {
        for (named, variants) in PRELUDE {
            let base = prelude_class(named);
            let mut constructors = Vec::new();
            for (position, (variant, carries)) in variants.iter().enumerate() {
                let held = (0..*carries)
                    .map(|position| Carried {
                        name: positional(position),
                        of: Some(object()),
                    })
                    .collect();
                let tag = i32::try_from(position).unwrap_or_default();
                constructors.push((*variant).to_owned());
                self.built_by.insert(
                    (*variant).to_owned(),
                    Shape {
                        class: variant_class(&base, variant),
                        base: base.clone(),
                        tag: Some(tag),
                        carries: held,
                    },
                );
            }
            self.declarations
                .push(Declared::Variants { base, constructors });
        }
    }

    fn payload(&self, resolved: &ResolvedProgram, variant: &Variant) -> Vec<Carried> {
        match &variant.payload {
            VariantPayload::None => Vec::new(),
            VariantPayload::Tuple(written) => written
                .iter()
                .enumerate()
                .map(|(position, type_ref)| Carried {
                    name: positional(position),
                    of: self.written(resolved, type_ref),
                })
                .collect(),
            VariantPayload::Record(fields) => self.fields(resolved, fields),
        }
    }

    fn fields(&self, resolved: &ResolvedProgram, written: &[RecordField]) -> Vec<Carried> {
        written
            .iter()
            .map(|field| Carried {
                name: field.name.text.clone(),
                of: self.written(resolved, &field.type_ref),
            })
            .collect()
    }

    /// What a type written in a declaration is carried by; a type parameter erases to `Object`.
    fn written(&self, resolved: &ResolvedProgram, type_ref: &TypeRef) -> Option<Descriptor> {
        let TypeRefKind::Named { name, .. } = &type_ref.kind else {
            return None;
        };
        let parameter = resolved
            .definition(Namespace::Type, name)
            .is_some_and(|definition| definition.kind == DefinitionKind::TypeParameter);
        if parameter {
            return Some(object());
        }
        Some(self.named(&name.text))
    }

    /// The class `constructor` builds, which name resolution has already proved is declared.
    pub(crate) fn built(&self, constructor: &str) -> &Shape {
        self.built_by
            .get(constructor)
            .expect("name resolution gave every constructor a declaration")
    }

    /// What a value of the type called `named` is carried by.
    fn named(&self, named: &str) -> Descriptor {
        match named {
            "Int" => Descriptor::Long,
            "Bool" => Descriptor::Boolean,
            "String" => Descriptor::reference("java/lang/String"),
            "List" => Descriptor::reference("java/util/List"),
            "Option" | "Result" => Descriptor::Reference(prelude_class(named)),
            declared => Descriptor::Reference(self.declared(declared)),
        }
    }

    /// The class written for the type called `declared`, which is of the module's package.
    fn declared(&self, declared: &str) -> ClassName {
        ClassName::new(&format!("{}/{declared}", self.module))
    }
}

/// The class written for a variant, which is the type's class and the variant's name.
///
/// A type and one of its variants may share a name, and two variants of two types may too, so
/// the type's class is part of the variant's class the way a nested class of Java is.
fn variant_class(base: &ClassName, variant: &str) -> ClassName {
    ClassName::new(&format!("{base}${variant}"))
}

fn prelude_class(named: &str) -> ClassName {
    ClassName::new(&format!("{PRELUDE_PACKAGE}/{named}"))
}

/// What a variant that carries its values in order calls the one at `position`.
fn positional(position: usize) -> String {
    format!("value{position}")
}

/// `java.lang.Object`, which is what every type a declaration leaves open erases to.
pub(crate) fn object() -> Descriptor {
    Descriptor::reference("java/lang/Object")
}
