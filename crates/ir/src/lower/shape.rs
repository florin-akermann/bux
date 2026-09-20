//! What each type a module declares is laid out as.
//!
//! Every constructor the source can write builds one class, and every type it can write is
//! carried by one descriptor. `docs/specs/codegen.md` states the layout; this is where a name
//! written in Lumen becomes a name the JVM knows.

use std::collections::{HashMap, HashSet};

use lumen_ast::{Item, RecordField, TypeDeclaration, TypeDefinition, TypeRef, TypeRefKind};
use lumen_ast::{Variant, VariantPayload};
use lumen_resolver::{DefinitionKind, Namespace, ResolvedProgram, prelude};
use lumen_types::{BuiltBy, OfferedConstructor, OfferedType, Type};

use crate::descriptor::{ClassName, Descriptor, MethodDescriptor};

/// The type a value of, or nothing, is written as, which a `?` hands a `None` back from.
pub(crate) const OPTION: &str = "Option";

/// The type an attempt is written as, which a `?` hands an `Err` back from.
pub(crate) const RESULT: &str = "Result";

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
    /// The types this module declares itself, which are the ones another module reaches by
    /// writing this module's name in front.
    own: HashSet<String>,
}

impl Shapes {
    /// The shapes `resolved` declares and `reached` offers, with the prelude beside them.
    pub(crate) fn of(resolved: &ResolvedProgram, module: &str, reached: &[OfferedType]) -> Self {
        let mut shapes = Self {
            module: ClassName::new(module),
            declarations: Vec::new(),
            built_by: HashMap::new(),
            records: HashMap::new(),
            own: HashSet::new(),
        };
        for item in &resolved.program().items {
            if let Item::Type(declaration) = item {
                shapes.declare(resolved, declaration);
            }
        }
        for offered in reached {
            shapes.declare_reached(offered);
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

    /// Whether `named` is the constructor of a record type, rather than of a variant of one.
    pub(crate) fn builds_a_record(&self, named: &str) -> bool {
        self.records
            .values()
            .any(|constructor| constructor == named)
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

    /// `of` written as the module `by` writes it, which is what asking `by` for a method needs.
    ///
    /// `docs/specs/codegen.md` states the two sides of it: a type `by` declares drops the module
    /// name this one reaches it through, and a type this module declares gains this module's.
    /// A type of the prelude, and a type of a third module, are written the same way in both.
    pub(crate) fn as_written_by(&self, by: &str, of: &Type) -> Type {
        let Type::Named { name, arguments } = of else {
            return of.clone();
        };
        Type::Named {
            name: self.written_by(by, name),
            arguments: arguments
                .iter()
                .map(|argument| self.as_written_by(by, argument))
                .collect(),
        }
    }

    /// The name `by` writes the type this module writes as `named` by.
    fn written_by(&self, by: &str, named: &str) -> String {
        if let Some((module, own)) = named.split_once('.') {
            return if module == by {
                own.to_owned()
            } else {
                named.to_owned()
            };
        }
        if self.own.contains(named) {
            return format!("{}.{named}", self.module);
        }
        named.to_owned()
    }

    fn declare(&mut self, resolved: &ResolvedProgram, declaration: &TypeDeclaration) {
        self.own.insert(declaration.name.text.clone());
        let base = self.declared(&declaration.name.text);
        self.declare_as(resolved, declaration, base);
    }

    /// A type of another module, whose classes that module writes and this one only reaches.
    ///
    /// Its shapes go nowhere near `declarations`: the module declaring it writes its classes,
    /// and writing them again here would be two modules claiming one name.
    fn declare_reached(&mut self, offered: &OfferedType) {
        let base = self.declared(offered.name());
        match offered.built_by() {
            BuiltBy::Record(one) => {
                self.records
                    .insert(offered.name().to_owned(), one.name().to_owned());
                let carries = self.offered_carries(one);
                self.built_by.insert(
                    one.name().to_owned(),
                    Shape {
                        class: base.clone(),
                        base,
                        tag: None,
                        carries,
                    },
                );
            }
            BuiltBy::Variants(variants) => {
                for (position, variant) in variants.iter().enumerate() {
                    let carries = self.offered_carries(variant);
                    let class = variant_class(&base, simple(variant.name()));
                    self.built_by.insert(
                        variant.name().to_owned(),
                        Shape {
                            class,
                            base: base.clone(),
                            tag: Some(i32::try_from(position).unwrap_or_default()),
                            carries,
                        },
                    );
                }
            }
        }
    }

    /// What one constructor of another module's type holds, named as that module named it.
    fn offered_carries(&self, built: &OfferedConstructor) -> Vec<Carried> {
        built
            .carries()
            .iter()
            .enumerate()
            .map(|(position, of)| Carried {
                name: built
                    .labels()
                    .get(position)
                    .cloned()
                    .unwrap_or_else(|| positional(position)),
                of: self.carried(of),
            })
            .collect()
    }

    /// The prelude's own types, which are in scope in every module without being declared.
    ///
    /// They are laid out exactly as a module's own types are, from the same declarations, and
    /// differ only in the package they are written in: every module reaches them, so no one
    /// module's package could hold them.
    fn declare_prelude(&mut self) {
        let resolved = lumen_resolver::prelude_resolved();
        for item in &resolved.program().items {
            let Item::Type(declaration) = item else {
                continue;
            };
            let base = prelude_class(&declaration.name.text);
            self.declare_as(resolved, declaration, base);
        }
    }

    /// One type declaration, laid out as the class `base` and one class for each variant.
    fn declare_as(
        &mut self,
        resolved: &ResolvedProgram,
        declaration: &TypeDeclaration,
        base: ClassName,
    ) {
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
        let TypeRefKind::Named { path, .. } = &type_ref.kind else {
            return None;
        };
        let parameter = path.module.is_none()
            && resolved
                .definition(Namespace::Type, &path.name)
                .is_some_and(|definition| definition.kind == DefinitionKind::TypeParameter);
        if parameter {
            return Some(object());
        }
        Some(self.named(&path.to_string()))
    }

    /// The class `constructor` builds, which name resolution has already proved is declared.
    pub(crate) fn built(&self, constructor: &str) -> &Shape {
        self.built_by
            .get(constructor)
            .expect("name resolution gave every constructor a declaration")
    }

    /// The class `name` builds inside `module`, when that module offers a constructor of it.
    pub(crate) fn offered(&self, module: &str, name: &str) -> Option<&Shape> {
        self.built_by.get(&format!("{module}.{name}"))
    }

    /// What a value of the type called `named` is carried by.
    fn named(&self, named: &str) -> Descriptor {
        match named {
            "Int" => Descriptor::Long,
            "Bool" => Descriptor::Boolean,
            "String" => Descriptor::reference("java/lang/String"),
            "List" => Descriptor::reference("java/util/List"),
            declared if prelude::declares_type(declared) => {
                Descriptor::Reference(prelude_class(declared))
            }
            declared => Descriptor::Reference(self.declared(declared)),
        }
    }

    /// The class written for the type called `declared`, which is of its own module's package.
    ///
    /// A name holding a dot is a type of another module, written `demo.User` as
    /// `docs/specs/modules.md` states, and the class it names is that module's own.
    fn declared(&self, declared: &str) -> ClassName {
        let Some((module, named)) = declared.split_once('.') else {
            return ClassName::new(&format!("{}/{declared}", self.module));
        };
        ClassName::new(&format!("{module}/{named}"))
    }
}

/// A name as the module declaring it wrote it, which is what is left of a dot where one is.
fn simple(reached: &str) -> &str {
    reached.split_once('.').map_or(reached, |(_, named)| named)
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
    Descriptor::Reference(object_class())
}

/// The class `java.lang.Object` is, which an array of open values is made of.
pub(crate) fn object_class() -> ClassName {
    ClassName::new("java/lang/Object")
}
