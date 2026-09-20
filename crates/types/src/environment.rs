//! What a module declares, gathered before any body is inferred.

use std::collections::{HashMap, HashSet};

use lumen_ast::{DeriveDeclaration, Function, InstanceDeclaration, TraitDeclaration};
use lumen_ast::{Item, Name, RecordField, Signature, Span, TypeDeclaration, TypeDefinition};
use lumen_ast::{TypeRef, TypeRefKind};
use lumen_ast::{Variant, VariantPayload};
use lumen_resolver::prelude;
use lumen_resolver::{Definition, DefinitionKind, Namespace, Origin, ResolvedProgram};

use crate::bounds::{self, Bounds};
use crate::derive;
use crate::error::{Count, TypeError, TypeErrorKind};
use crate::scheme::{Quantified, Required, Scheme};
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
    /// The stand-in each trait method's scheme is written over, which is its trait's parameter.
    methods: HashMap<Key, Quantified>,
    /// Every trait that has an instance for a type, by the two names that say so.
    instances: HashSet<(String, String)>,
    /// What each type's instance of `IntegerLiteral` says it holds, by the name of that type.
    holds: HashMap<String, Bounds>,
    /// What each instance's method must be, which is its trait's method at the instance's type.
    written_as: HashMap<Key, Type>,
    /// Where each method of each trait is declared, which is what an instance is read against.
    declares: HashMap<(String, String), Key>,
}

impl Environment {
    /// Everything `resolved` declares, with the prelude already in it.
    ///
    /// # Errors
    ///
    /// Returns the first written type that names the wrong number of arguments.
    pub(crate) fn of(resolved: &ResolvedProgram, table: &mut Table) -> Result<Self, TypeError> {
        let mut environment = Self::of_prelude();
        environment.note_arities(resolved);
        environment.declare_traits(resolved)?;
        for item in &resolved.program().items {
            environment.declare(resolved, item, table)?;
        }
        derive::compare_what_they_hold(&environment, resolved)?;
        Ok(environment)
    }

    /// How many arguments each declared type takes, which every written type is then held to.
    ///
    /// This is read while a declaration is being read, so it is gathered before any of them are.
    fn note_arities(&mut self, resolved: &ResolvedProgram) {
        for item in &resolved.program().items {
            let Item::Type(declaration) = item else {
                continue;
            };
            self.arities
                .insert(Key::at(&declaration.name), declaration.parameters.len());
        }
    }

    /// Every trait the module declares, which an instance below it is read against.
    ///
    /// # Errors
    ///
    /// Returns the first written type that names the wrong number of arguments.
    fn declare_traits(&mut self, resolved: &ResolvedProgram) -> Result<(), TypeError> {
        for item in &resolved.program().items {
            let Item::Trait(declaration) = item else {
                continue;
            };
            self.declare_trait(resolved, declaration)?;
        }
        Ok(())
    }

    /// The stand-in the method defined at `key` is written over, when it is a trait's method.
    ///
    /// Only a trait's method has one, so this is also what says a name is one of them.
    pub(crate) fn of_a_trait(&self, key: &Key) -> Option<&Quantified> {
        self.methods.get(key)
    }

    /// The whole numbers the type called `for_type` holds, when it takes a literal at all.
    pub(crate) fn holds(&self, for_type: &str) -> Option<Bounds> {
        self.holds.get(for_type).copied()
    }

    /// The types a whole number may be written at, which is every one with the instance.
    pub(crate) fn takes_a_whole_number(&self) -> impl Iterator<Item = String> + '_ {
        self.holds.keys().cloned()
    }

    /// Whether the trait called `of` has an instance for the type called `for_type`.
    pub(crate) fn has_instance(&self, of: &str, for_type: &str) -> bool {
        self.instances
            .contains(&(of.to_owned(), for_type.to_owned()))
    }

    /// What the instance method defined at `key` must be, when `key` is one of them.
    pub(crate) fn written_as(&self, key: &Key) -> Option<&Type> {
        self.written_as.get(key)
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
        environment.bind(Key::prelude("todo"), todo(&value));
        environment.supply_traits(&value);
        environment
    }

    /// The traits the prelude supplies, and the instances the library will ship for each.
    ///
    /// `docs/specs/traits.md` writes `Eq` out and `docs/specs/operators.md` writes the trait each
    /// operator is. They are declared here for the reason the prelude's types are declared here:
    /// a module cannot be loaded from a file yet.
    fn supply_traits(&mut self, value: &TypeParameter) {
        for supplied in &prelude::TRAITS {
            for method in supplied.methods {
                self.supply(value, supplied.name, method);
            }
        }
        for (of, for_type) in prelude::instances() {
            self.instances.insert((of.to_owned(), for_type.to_owned()));
        }
        self.holds.insert(Type::int().to_string(), Bounds::INT);
    }

    /// One method of one supplied trait, a name in scope with the type its trait gives it.
    ///
    /// The scheme is written over the trait's type parameter and asks that trait of it, so a use
    /// of the method is answered by an instance exactly as a use of a declared one is.
    fn supply(&mut self, value: &TypeParameter, of: &str, method: &str) {
        let key = Key::prelude(method);
        let at = Type::Parameter(value.clone());
        let asks = vec![Required {
            trait_name: of.to_owned(),
            at: at.clone(),
        }];
        let signature = signature_of(of, method, &at);
        self.bind(
            key.clone(),
            Scheme::over(vec![Quantified::Parameter(value.clone())], signature).requiring(asks),
        );
        self.methods
            .insert(key.clone(), Quantified::Parameter(value.clone()));
        self.declares
            .insert((of.to_owned(), method.to_owned()), key);
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
            Item::Trait(_) => Ok(()),
            Item::Instance(declaration) => self.declare_instance(resolved, declaration, table),
            Item::Derive(declaration) => self.declare_derive(resolved, declaration),
            Item::Function(function) => self.declare_function(resolved, function, table),
        }
    }

    /// A trait gives each of its methods a type, written over the one parameter it declares.
    ///
    /// The method is the name every instance answers for, so this is the type a call of it is
    /// read against, whichever instance turns out to answer.
    fn declare_trait(
        &mut self,
        resolved: &ResolvedProgram,
        declaration: &TraitDeclaration,
    ) -> Result<(), TypeError> {
        let written = TypeParameter::written(&declaration.parameter);
        let parameter = Quantified::Parameter(written.clone());
        let asks = vec![Required {
            trait_name: declaration.name.text.clone(),
            at: Type::Parameter(written),
        }];
        for method in &declaration.methods {
            let signature = self.signature(resolved, method)?;
            let key = Key::at(&method.name);
            let scheme = Scheme::over(vec![parameter.clone()], signature).requiring(asks.clone());
            self.bind(key.clone(), scheme);
            self.methods.insert(key.clone(), parameter.clone());
            self.declares.insert(
                (declaration.name.text.clone(), method.name.text.clone()),
                key,
            );
        }
        Ok(())
    }

    /// One signature of a trait, which names types and nothing that has to be inferred.
    ///
    /// A function leaves a parameter's type out and inference reads it off the body. A signature
    /// has no body, so a type left out here is one nothing would ever settle, and it is refused
    /// where it is missing rather than at the instance that is later held to whatever it became.
    fn signature(&self, resolved: &ResolvedProgram, method: &Signature) -> Result<Type, TypeError> {
        let mut parameters = Vec::new();
        for parameter in &method.parameters {
            let Some(written) = &parameter.type_ref else {
                let kind = TypeErrorKind::SignatureWithoutType(parameter.name.text.clone());
                return Err(TypeError::at(parameter.name.span, kind));
            };
            parameters.push(self.written(resolved, written)?);
        }
        let result = match &method.result {
            Some(written) => self.written(resolved, written)?,
            None => Type::Unit,
        };
        Ok(Type::function(parameters, result))
    }

    /// An instance: the methods it writes, and what each of them has to be.
    ///
    /// An instance's method is monomorphic — its trait's parameter has settled on the instance's
    /// type — so it is declared as a function is, and held to the trait's signature at that type.
    fn declare_instance(
        &mut self,
        resolved: &ResolvedProgram,
        declaration: &InstanceDeclaration,
        table: &mut Table,
    ) -> Result<(), TypeError> {
        self.takes_no_arguments(resolved, &declaration.for_type)?;
        let of = declaration.trait_name.text.clone();
        let for_type = declaration.for_type.text.clone();
        if of == prelude::INTEGER_LITERAL {
            self.holds
                .insert(for_type.clone(), bounds::stated(declaration)?);
        }
        self.instances.insert((of, for_type.clone()));
        let given = Type::Named {
            name: for_type,
            arguments: Vec::new(),
        };
        for method in &declaration.methods {
            self.declare_function(resolved, method, table)?;
            let of = &declaration.trait_name.text;
            let Some(declared) = self.method_at(of, &method.name.text, &given) else {
                continue;
            };
            self.written_as.insert(Key::at(&method.name), declared);
        }
        Ok(())
    }

    /// A derive: the instances it claims, each for the whole type it names.
    ///
    /// What each instance writes is settled by the declaration rather than by anything written
    /// here, so a derive declares the instance and leaves the checking of it to the pass that
    /// runs once every item has been declared.
    fn declare_derive(
        &mut self,
        resolved: &ResolvedProgram,
        declaration: &DeriveDeclaration,
    ) -> Result<(), TypeError> {
        self.takes_no_arguments(resolved, &declaration.for_type)?;
        for named in &declaration.traits {
            self.instances
                .insert((named.text.clone(), declaration.for_type.text.clone()));
        }
        Ok(())
    }

    /// An instance is for a whole type, so the type it names takes no arguments.
    ///
    /// `instance Eq<Option>` would be an instance of one name, and every `Option<T>` would then
    /// reach it whatever `T` turned out to be. An instance per argument waits for the spec that
    /// derives one, so a type that takes arguments is refused here, counted as anywhere else.
    ///
    /// # Errors
    ///
    /// Returns the instance whose type takes arguments that the instance does not name.
    fn takes_no_arguments(
        &self,
        resolved: &ResolvedProgram,
        for_type: &Name,
    ) -> Result<(), TypeError> {
        let definition = resolved
            .definition(Namespace::Type, for_type)
            .expect("name resolution gave the instance's type a definition");
        let key = Key::of(definition, for_type);
        Self::counted(for_type, *self.arities.get(&key).unwrap_or(&0), 0)
    }

    /// The type the trait `of` gives the method `method` at `given`, when it declares one.
    fn method_at(&self, of: &str, method: &str, given: &Type) -> Option<Type> {
        let key = self.declares.get(&(of.to_owned(), method.to_owned()))?;
        let scheme = self.values.get(key)?;
        let parameter = self.methods.get(key)?;
        Some(scheme.settling(parameter, given.clone()))
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
        let over = written_over(&function.type_parameters);
        let signature = Type::function(parameters, result);
        let required = self.constraints(resolved, function)?;
        let scheme = Scheme::over(over, signature).requiring(required);
        self.bind(Key::at(&function.name), scheme);
        Ok(())
    }

    /// The traits a function's type parameters are constrained by, at the types they constrain.
    fn constraints(
        &self,
        resolved: &ResolvedProgram,
        function: &Function,
    ) -> Result<Vec<Required>, TypeError> {
        let mut required = Vec::new();
        for parameter in &function.type_parameters {
            let Some(constraint) = &parameter.constraint else {
                continue;
            };
            required.push(Required {
                trait_name: constraint.name.text.clone(),
                at: self.written(resolved, &constraint.argument)?,
            });
        }
        Ok(required)
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

/// The type one method of one prelude trait has, written over that trait's own type parameter.
///
/// `docs/specs/operators.md` gives each operator's trait its signature: one gives back what it
/// was given, one an `Option` of it, and one a `Bool` about it. `docs/specs/literals.md` gives
/// `IntegerLiteral` its three, which are the only ones a prelude trait declares more than one of.
fn signature_of(of: &str, method: &str, at: &Type) -> Type {
    let two = vec![at.clone(), at.clone()];
    match of {
        prelude::EQ | prelude::ORD => Type::function(two, Type::boolean()),
        prelude::DIV | prelude::REM => Type::function(two, Type::option(at.clone())),
        prelude::NEG => Type::function(vec![at.clone()], at.clone()),
        prelude::ADD | prelude::SUB | prelude::MUL => Type::function(two, at.clone()),
        prelude::INTEGER_LITERAL => written_as_a_literal(method, at),
        _ => unreachable!("the prelude declares exactly the traits `prelude::TRAITS` lists"),
    }
}

/// The type one method of `IntegerLiteral` has: a bound gives an `Int`, and the third builds a `T`.
fn written_as_a_literal(method: &str, at: &Type) -> Type {
    if method == prelude::FROM_LITERAL {
        return Type::function(vec![Type::int()], at.clone());
    }
    Type::function(Vec::new(), Type::int())
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

/// The stand-ins a function is written over, which are its type parameters constrained or not.
fn written_over(parameters: &[lumen_ast::TypeParameter]) -> Vec<Quantified> {
    parameters
        .iter()
        .map(|parameter| Quantified::Parameter(TypeParameter::written(&parameter.name)))
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

/// `todo(reason)`: a hole, which is whatever type the place it is written in expects.
///
/// `docs/specs/holes.md` states what it is for. It gives back a type nothing constrains, so a
/// hole unifies with whatever belongs where it is written.
fn todo(value: &TypeParameter) -> Scheme {
    Scheme::over(
        vec![Quantified::Parameter(value.clone())],
        Type::function(vec![Type::string()], Type::Parameter(value.clone())),
    )
}
