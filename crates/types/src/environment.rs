//! What a module declares, gathered before any body is inferred.

use std::collections::HashMap;

use lumen_ast::{Function, TypeRef, TypeRefKind, Variant, VariantPayload};
use lumen_ast::{Item, Name, RecordField, Span, TypeDeclaration, TypeDefinition};
use lumen_resolver::{Definition, DefinitionKind, Namespace, Origin, ResolvedProgram};

use crate::error::{Count, TypeError, TypeErrorKind};
use crate::scheme::{Quantified, Scheme};
use crate::table::Table;
use crate::types::{Type, TypeParameter};

/// Every name that has a type, and the shape of every type that has fields.
///
/// A name is known by where it was defined rather than by how it is spelled, which is what lets
/// one flat table hold the whole module: name resolution has already ruled out two definitions
/// sharing a name and a binding hiding one.
#[derive(Debug, Default)]
pub(crate) struct Environment {
    values: HashMap<Key, Scheme>,
    labels: HashMap<Key, Vec<String>>,
    records: HashMap<String, Key>,
    arities: HashMap<Key, usize>,
}

impl Environment {
    /// Everything `resolved` declares, with the prelude already in it.
    ///
    /// # Errors
    ///
    /// Returns the first written type that names the wrong number of arguments.
    pub(crate) fn of(resolved: &ResolvedProgram, table: &mut Table) -> Result<Self, TypeError> {
        let mut environment = Self::of_prelude();
        for item in &resolved.program().items {
            if let Item::Type(declaration) = item {
                let arity = declaration.parameters.len();
                environment
                    .arities
                    .insert(Key::at(&declaration.name), arity);
            }
        }
        for item in &resolved.program().items {
            environment.declare(resolved, item, table)?;
        }
        Ok(environment)
    }

    /// The type of the name defined at `key`, when that name has one.
    pub(crate) fn scheme(&self, key: &Key) -> Option<&Scheme> {
        self.values.get(key)
    }

    /// Takes the name defined at `key` back out, which is what leaving its scope amounts to.
    pub(crate) fn unbind(&mut self, key: &Key) -> Option<Scheme> {
        self.values.remove(key)
    }

    /// The labels the constructor defined at `key` takes, in the order it takes them.
    pub(crate) fn labels(&self, key: &Key) -> &[String] {
        self.labels.get(key).map_or(&[], Vec::as_slice)
    }

    /// The constructor of the record type called `name`, when `name` is a record type.
    pub(crate) fn record(&self, name: &str) -> Option<&Key> {
        self.records.get(name)
    }

    /// Every scheme in the table, which is what a generalisation must not quantify over.
    pub(crate) fn schemes(&self) -> impl Iterator<Item = &Scheme> {
        self.values.values()
    }

    fn of_prelude() -> Self {
        let mut environment = Self::default();
        for (name, arity) in PRELUDE_TYPES {
            environment.arities.insert(Key::prelude(name), arity);
        }
        let value = TypeParameter::prelude("T");
        let error = TypeParameter::prelude("E");
        let option = Type::option(Type::Parameter(value.clone()));
        let result = Type::result(
            Type::Parameter(value.clone()),
            Type::Parameter(error.clone()),
        );
        let over_value = vec![Quantified::Parameter(value.clone())];
        let over_both = vec![
            Quantified::Parameter(value.clone()),
            Quantified::Parameter(error.clone()),
        ];
        let carried = Type::Parameter(value.clone());
        environment.bind(
            Key::prelude("None"),
            Scheme::over(over_value.clone(), option.clone()),
        );
        environment.bind(
            Key::prelude("Some"),
            Scheme::over(over_value, Type::function(vec![carried.clone()], option)),
        );
        environment.bind(
            Key::prelude("Ok"),
            Scheme::over(
                over_both.clone(),
                Type::function(vec![carried], result.clone()),
            ),
        );
        environment.bind(
            Key::prelude("Err"),
            Scheme::over(
                over_both,
                Type::function(vec![Type::Parameter(error)], result),
            ),
        );
        environment.bind(Key::prelude("or"), or(&value));
        environment
    }

    fn declare(
        &mut self,
        resolved: &ResolvedProgram,
        item: &Item,
        table: &mut Table,
    ) -> Result<(), TypeError> {
        match item {
            Item::Import(import) => {
                let module = Type::Module(import.module.text.clone());
                self.bind(Key::at(&import.module), Scheme::monomorphic(module));
                Ok(())
            }
            Item::Type(declaration) => self.declare_type(resolved, declaration),
            Item::Function(function) => self.declare_function(resolved, function, table),
        }
    }

    /// A type declaration gives a type to each of the values its variants are built with.
    fn declare_type(
        &mut self,
        resolved: &ResolvedProgram,
        declaration: &TypeDeclaration,
    ) -> Result<(), TypeError> {
        let over = quantified(&declaration.parameters);
        let declared = Type::Named {
            name: declaration.name.text.clone(),
            arguments: declaration
                .parameters
                .iter()
                .map(|parameter| Type::Parameter(TypeParameter::written(parameter)))
                .collect(),
        };
        let TypeDefinition::Variants(variants) = &declaration.definition else {
            let TypeDefinition::Record(fields) = &declaration.definition else {
                unreachable!("a type declaration is a record or variants")
            };
            self.records
                .insert(declaration.name.text.clone(), Key::at(&declaration.name));
            let built = self.built_from(resolved, &declaration.name, fields)?;
            self.build_with(built, &over, &declared);
            return Ok(());
        };
        for variant in variants {
            let built = self.built_variant(resolved, variant)?;
            self.build_with(built, &over, &declared);
        }
        Ok(())
    }

    /// A function's type as its signature reads, with a variable where nothing was written.
    fn declare_function(
        &mut self,
        resolved: &ResolvedProgram,
        function: &Function,
        table: &mut Table,
    ) -> Result<(), TypeError> {
        let mut parameters = Vec::new();
        for parameter in &function.parameters {
            parameters.push(match &parameter.type_ref {
                Some(written) => self.written(resolved, written)?,
                None => table.fresh(),
            });
        }
        let result = match &function.result {
            Some(written) => self.written(resolved, written)?,
            None => table.fresh(),
        };
        let over = quantified(&function.type_parameters);
        let signature = Type::function(parameters, result);
        self.bind(Key::at(&function.name), Scheme::over(over, signature));
        Ok(())
    }

    fn built_variant(
        &self,
        resolved: &ResolvedProgram,
        variant: &Variant,
    ) -> Result<Built, TypeError> {
        match &variant.payload {
            VariantPayload::None => Ok(Built {
                key: Key::at(&variant.name),
                labels: Vec::new(),
                carries: Vec::new(),
            }),
            VariantPayload::Tuple(written) => Ok(Built {
                key: Key::at(&variant.name),
                labels: Vec::new(),
                carries: self.each(resolved, written)?,
            }),
            VariantPayload::Record(fields) => self.built_from(resolved, &variant.name, fields),
        }
    }

    fn built_from(
        &self,
        resolved: &ResolvedProgram,
        name: &Name,
        fields: &[RecordField],
    ) -> Result<Built, TypeError> {
        let mut labels = Vec::new();
        let mut carries = Vec::new();
        for field in fields {
            labels.push(field.name.text.clone());
            carries.push(self.written(resolved, &field.type_ref)?);
        }
        Ok(Built {
            key: Key::at(name),
            labels,
            carries,
        })
    }

    fn build_with(&mut self, built: Built, over: &[Quantified], declared: &Type) {
        let Built {
            key,
            labels,
            carries,
        } = built;
        let body = if carries.is_empty() {
            declared.clone()
        } else {
            Type::function(carries, declared.clone())
        };
        self.labels.insert(key.clone(), labels);
        self.bind(key, Scheme::over(over.to_vec(), body));
    }

    /// Gives the name defined at `key` the type `scheme`.
    pub(crate) fn bind(&mut self, key: Key, scheme: Scheme) {
        self.values.insert(key, scheme);
    }

    /// `written` as a type, with each name it holds pointed at what it was resolved to.
    ///
    /// # Errors
    ///
    /// Returns the first name written with the wrong number of type arguments.
    pub(crate) fn written(
        &self,
        resolved: &ResolvedProgram,
        written: &TypeRef,
    ) -> Result<Type, TypeError> {
        let TypeRefKind::Named { name, arguments } = &written.kind else {
            return Ok(Type::Unit);
        };
        let definition = resolved
            .definition(Namespace::Type, name)
            .expect("name resolution gave every written type a definition");
        if definition.kind == DefinitionKind::TypeParameter {
            Self::counted(name, 0, arguments.len())?;
            return Ok(Type::Parameter(parameter_of(definition, name)));
        }
        let key = Key::of(definition, name);
        Self::counted(name, *self.arities.get(&key).unwrap_or(&0), arguments.len())?;
        Ok(Type::Named {
            name: name.text.clone(),
            arguments: self.each(resolved, arguments)?,
        })
    }

    fn each(
        &self,
        resolved: &ResolvedProgram,
        written: &[TypeRef],
    ) -> Result<Vec<Type>, TypeError> {
        written
            .iter()
            .map(|one| self.written(resolved, one))
            .collect()
    }

    fn counted(name: &Name, takes: usize, given: usize) -> Result<(), TypeError> {
        if takes == given {
            return Ok(());
        }
        let count = Count {
            name: name.text.clone(),
            takes,
            given,
        };
        Err(TypeError::at(
            name.span,
            TypeErrorKind::WrongTypeArgumentCount(count),
        ))
    }
}

/// Where a name was defined, which is what a type is filed under.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Key {
    /// A name this module declares, known by the place it is declared.
    Declared(Span),
    /// A name the prelude supplies, known by the one spelling it has.
    Prelude(String),
}

impl Key {
    /// The key of a use of `name`, which `definition` says where to find.
    pub(crate) fn of(definition: Definition, name: &Name) -> Self {
        match definition.origin {
            Origin::Declared(span) => Self::Declared(span),
            Origin::Prelude => Self::Prelude(name.text.clone()),
        }
    }

    /// The key of the declaration `name` is the name of.
    pub(crate) const fn at(name: &Name) -> Self {
        Self::Declared(name.span)
    }

    fn prelude(name: &str) -> Self {
        Self::Prelude(name.to_owned())
    }
}

/// The types the prelude supplies, with how many arguments each one is written with.
const PRELUDE_TYPES: [(&str, usize); 6] = [
    ("Bool", 0),
    ("Int", 0),
    ("List", 1),
    ("Option", 1),
    ("Result", 2),
    ("String", 0),
];

/// One constructor: where it is declared, what it labels, and what it carries.
struct Built {
    key: Key,
    labels: Vec<String>,
    carries: Vec<Type>,
}

fn quantified(parameters: &[Name]) -> Vec<Quantified> {
    parameters
        .iter()
        .map(|parameter| Quantified::Parameter(TypeParameter::written(parameter)))
        .collect()
}

fn parameter_of(definition: Definition, name: &Name) -> TypeParameter {
    TypeParameter {
        name: name.text.clone(),
        origin: definition.origin,
    }
}

/// `or(maybe, fallback)`: what an `Option` holds, or the fallback when it holds nothing.
///
/// `Result` has no `or` yet, because two functions of one name wait on typeclasses, which
/// `docs/implementation.md` section 9 leaves out of version 0.1.
fn or(value: &TypeParameter) -> Scheme {
    let held = Type::Parameter(value.clone());
    Scheme::over(
        vec![Quantified::Parameter(value.clone())],
        Type::function(vec![Type::option(held.clone()), held.clone()], held),
    )
}
