//! What a module declares, gathered before any body is inferred.

use std::collections::HashMap;

use lumen_ast::{Called, TraitDeclaration};
use lumen_ast::{DeriveDeclaration, ExternDeclaration, Function, InstanceDeclaration};
use lumen_ast::{Item, Name, Path, RecordField, Signature, TypeDeclaration, TypeDefinition};
use lumen_ast::{TypeRef, TypeRefKind};
use lumen_ast::{Variant, VariantPayload};
use lumen_resolver::prelude;
use lumen_resolver::{DefinitionKind, Namespace, ResolvedProgram};

use crate::boundary::{self, Crossing};
use crate::bounds::{self, Bounds};
use crate::derive;
use crate::error::{Count, TypeError, TypeErrorKind};
use crate::held;
use crate::scheme::{Quantified, Required, Scheme};
use crate::surface::{BuiltBy, OfferedType};
use crate::table::Table;
use crate::types::{Type, TypeParameter};

mod carried;
mod key;

use carried::Keyed;
pub(crate) use carried::{prelude_checked, signature_of};
pub(crate) use key::Key;
use key::{Built, parameter_of, quantified, written_over};

/// Every name that has a type, and the shape of every type that has fields.
///
/// A name is known by where it was defined rather than by how it is spelled, which is what lets
/// one flat table hold the whole module: name resolution has already ruled out two definitions
/// sharing a name and a binding hiding one.
#[derive(Clone, Debug, Default)]
pub(crate) struct Environment {
    /// How a declaration of the source being read is filed, which the prelude differs in.
    keyed: Keyed,
    values: HashMap<Key, Scheme>,
    labels: HashMap<Key, Vec<String>>,
    records: HashMap<String, Key>,
    arities: HashMap<Key, usize>,
    /// The stand-in each trait method's scheme is written over, which is its trait's parameter.
    methods: HashMap<Key, Quantified>,
    /// Every trait that has an instance for a type, by the two names that say so, and what the
    /// instance asks of each argument the type is written with.
    ///
    /// An instance over a type written with arguments is constrained on them, which
    /// `docs/specs/traits.md` states, so `Eq<List<T>>` at `T: Eq<T>` is filed with one ask of
    /// `Eq`. An argument the instance leaves unconstrained is filed with no ask at all.
    instances: HashMap<(String, String), Vec<Vec<String>>>,
    /// What each type's instance of `IntegerLiteral` says it holds, by the name of that type.
    holds: HashMap<String, Bounds>,
    /// What each instance's method must be, which is its trait's method at the instance's type.
    written_as: HashMap<Key, Type>,
    /// Where each method of each trait is declared, which is what an instance is read against.
    declares: HashMap<(String, String), Key>,
    /// The type each declaration declares, over its own parameters, by the name it declares it.
    declared: HashMap<String, Scheme>,
    /// How a method of each extern type is called, by the Lumen name an `extern type` gives it.
    foreign: HashMap<String, Called>,
}

impl Environment {
    /// Everything `resolved` declares, with the prelude already in it.
    ///
    /// # Errors
    ///
    /// Returns the first written type that names the wrong number of arguments.
    pub(crate) fn of(
        resolved: &ResolvedProgram,
        reached: &[OfferedType],
        table: &mut Table,
    ) -> Result<Self, TypeError> {
        let mut environment = Self::of_prelude();
        environment.held_by_the_compiler(resolved.module());
        for offered in reached {
            environment.offered_by_another_module(offered);
        }
        environment.note_types(resolved);
        environment.declare_traits(resolved)?;
        environment.declare_items(resolved, table)?;
        derive::hold_what_they_need(&environment, resolved)?;
        Ok(environment)
    }

    /// The functions the compiler holds for this module, under the names it writes them by.
    ///
    /// `docs/specs/library.md` says `push` and `at` are `list`'s names although the compiler
    /// holds them, so `list`'s own source writes them exactly as it writes `length`. A module
    /// reaches a name it does not declare by its spelling, which is what files them here.
    fn held_by_the_compiler(&mut self, module: &str) {
        for (name, scheme) in held::held_for(module) {
            self.bind(Key::prelude(name), scheme);
        }
    }

    /// One type another module offers, under the name a module importing it writes it by.
    ///
    /// It is declared exactly as a type of this module is: the same arity, the same constructors,
    /// and the same labels, because it is that module's type and not a copy of it.
    fn offered_by_another_module(&mut self, offered: &OfferedType) {
        let named = offered.name().to_owned();
        self.arities.insert(Key::reached(&named), offered.arity());
        match offered.built_by() {
            BuiltBy::Record(built) => {
                self.records
                    .insert(named.clone(), Key::reached(built.name()));
            }
            BuiltBy::Foreign { called, .. } => {
                self.foreign.insert(named.clone(), *called);
            }
            BuiltBy::Variants(_) => {}
        }
        for built in offered.constructors() {
            let key = Key::reached(built.name());
            self.labels.insert(key.clone(), built.labels().to_vec());
            self.bind(key, offered.scheme_of(built));
        }
    }

    /// What each declared type is, before any declaration that writes one is read.
    ///
    /// How many arguments it takes is what every written type is held to, and the kind of class
    /// it names is what an `extern` signature naming it crosses as. A file reads top down and
    /// a definition sits below what uses it, so both are gathered ahead of the declarations.
    fn note_types(&mut self, resolved: &ResolvedProgram) {
        for item in &resolved.program().items {
            let Item::Type(declaration) = item else {
                continue;
            };
            self.arities.insert(
                self.declared_at(&declaration.name),
                declaration.parameters.len(),
            );
            if let TypeDefinition::Foreign { called, .. } = &declaration.definition {
                self.foreign.insert(declaration.name.text.clone(), *called);
            }
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

    /// Whether the trait called `of` is answered at `at`, which is what a constraint resolves to.
    ///
    /// A named type reaches the one instance of that trait for it, and an instance over a type
    /// written with arguments asks its own constraint of each of them. So `Eq` at `List<Int>` is
    /// the one `Eq<List<T>>` there is, answered because `Eq` is answered at `Int`.
    ///
    /// A type parameter is answered by the constraints `promised` holds and by nothing else, which
    /// is what has `Eq` answered at `List<T>` inside a body written over `T: Eq<T>`. Anything else
    /// names no instance at all. `docs/specs/traits.md` states the three cases.
    pub(crate) fn answers(&self, of: &str, at: &Type, promised: &[Required]) -> bool {
        match at {
            Type::Named { name, arguments } => self
                .instances
                .get(&(of.to_owned(), name.clone()))
                .is_some_and(|asks| self.each_argument_answers(asks, arguments, promised)),
            Type::Parameter(_) => promised
                .iter()
                .any(|promise| promise.trait_name == *of && promise.at == *at),
            _ => false,
        }
    }

    /// Whether every argument of a type answers what the instance asks of it, where it asks.
    fn each_argument_answers(
        &self,
        asks: &[Vec<String>],
        arguments: &[Type],
        promised: &[Required],
    ) -> bool {
        asks.iter()
            .zip(arguments)
            .all(|(asked, argument)| asked.iter().all(|of| self.answers(of, argument, promised)))
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

    /// The type the declaration called `name` declares, written over its own parameters.
    pub(crate) fn declared_as(&self, name: &str) -> Option<&Scheme> {
        self.declared.get(name)
    }

    /// The constructor of the record type called `name`, when `name` is a record type.
    pub(crate) fn record(&self, name: &str) -> Option<&Key> {
        self.records.get(name)
    }

    /// Every scheme in the table, which is what a generalisation must not quantify over.
    pub(crate) fn schemes(&self) -> impl Iterator<Item = &Scheme> {
        self.values.values()
    }

    /// Every item of `resolved`, declared in the order it is written.
    ///
    /// # Errors
    ///
    /// Returns the first declaration that does not hold together.
    fn declare_items(
        &mut self,
        resolved: &ResolvedProgram,
        table: &mut Table,
    ) -> Result<(), TypeError> {
        for item in &resolved.program().items {
            self.declare(resolved, item, table)?;
        }
        Ok(())
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
                self.bind(
                    self.declared_at(&import.module),
                    Scheme::monomorphic(module),
                );
                Ok(())
            }
            Item::Type(declaration) => self.declare_type(resolved, declaration),
            Item::Trait(_) => Ok(()),
            Item::Instance(declaration) => self.declare_instance(resolved, declaration, table),
            Item::Derive(declaration) => self.declare_derive(resolved, declaration),
            Item::Function(function) => self.declare_function(resolved, function, table),
            Item::Extern(declaration) => self.declare_extern(resolved, declaration),
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
            let key = self.declared_at(&method.name);
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
        self.note_instance(resolved, declaration)?;
        let given = Type::Named {
            name: declaration.for_type.text.clone(),
            arguments: standing_for(declaration),
        };
        for method in &declaration.methods {
            self.declare_function(resolved, method, table)?;
            let of = &declaration.trait_name.text;
            let Some(declared) = self.method_at(of, &method.name.text, &given) else {
                continue;
            };
            self.written_as
                .insert(self.declared_at(&method.name), declared);
        }
        Ok(())
    }

    /// That an instance exists, and what it says its type holds when it is an `IntegerLiteral`.
    ///
    /// This is what a module reaching the instance asks for; what the instance writes to answer
    /// with is declared beside it, where there is a body to declare.
    ///
    /// # Errors
    ///
    /// Returns the instance whose type takes arguments, or whose stated bounds are not numbers.
    fn note_instance(
        &mut self,
        resolved: &ResolvedProgram,
        declaration: &InstanceDeclaration,
    ) -> Result<(), TypeError> {
        self.takes_its_own_arguments(resolved, declaration)?;
        let of = declaration.trait_name.text.clone();
        let for_type = declaration.for_type.text.clone();
        if of == prelude::INTEGER_LITERAL {
            self.holds
                .insert(for_type.clone(), bounds::stated(declaration)?);
        }
        self.instances.insert((of, for_type), asked_of(declaration));
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
            self.instances.insert(
                (named.text.clone(), declaration.for_type.text.clone()),
                Vec::new(),
            );
        }
        Ok(())
    }

    /// A derive is for a whole type, so the type it names takes no arguments.
    ///
    /// `derive Eq for Option` would be a derive of one name, and every `Option<T>` would then
    /// reach it whatever `T` turned out to be. What a derive writes reads what the type holds,
    /// and a type parameter holds nothing to read, so a type that takes arguments is refused
    /// here, counted as anywhere else.
    ///
    /// # Errors
    ///
    /// Returns the derive whose type takes arguments that the derive does not name.
    fn takes_no_arguments(
        &self,
        resolved: &ResolvedProgram,
        for_type: &Name,
    ) -> Result<(), TypeError> {
        self.counting(resolved, for_type, 0)
    }

    /// An instance is written with one argument for each its type takes, and each one is a type
    /// parameter it declares, which name resolution has already held it to.
    ///
    /// # Errors
    ///
    /// Returns the instance that writes more or fewer arguments than its type takes.
    fn takes_its_own_arguments(
        &self,
        resolved: &ResolvedProgram,
        declaration: &InstanceDeclaration,
    ) -> Result<(), TypeError> {
        self.counting(resolved, &declaration.for_type, declaration.arguments.len())
    }

    /// Holds the type called `for_type` to being written with `given` arguments.
    fn counting(
        &self,
        resolved: &ResolvedProgram,
        for_type: &Name,
        given: usize,
    ) -> Result<(), TypeError> {
        let definition = resolved
            .definition(Namespace::Type, for_type)
            .expect("name resolution gave the instance's type a definition");
        let key = self.key_of(definition, for_type);
        Self::counted(for_type, *self.arities.get(&key).unwrap_or(&0), given)
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
        self.declared.insert(
            declaration.name.text.clone(),
            Scheme::over(over.clone(), declared.clone()),
        );
        match &declaration.definition {
            TypeDefinition::Foreign { class, .. } => boundary::class(class)?,
            TypeDefinition::Record(fields) => {
                self.records.insert(
                    declaration.name.text.clone(),
                    self.declared_at(&declaration.name),
                );
                let built = self.built_from(resolved, &declaration.name, fields)?;
                self.build_with(built, &over, &declared);
            }
            TypeDefinition::Variants(variants) => {
                for variant in variants {
                    let built = self.built_variant(resolved, variant)?;
                    self.build_with(built, &over, &declared);
                }
            }
        }
        Ok(())
    }

    /// An `extern`'s type as its signature reads, every part of it written out.
    ///
    /// There is no body below it for inference to read anything off, so every parameter states
    /// its type and so does the result. What each of them may be is the boundary's own rule,
    /// which `docs/specs/interop.md` states and `crate::boundary` holds it to.
    fn declare_extern(
        &mut self,
        resolved: &ResolvedProgram,
        declaration: &ExternDeclaration,
    ) -> Result<(), TypeError> {
        let mut parameters = Vec::new();
        for parameter in &declaration.parameters {
            let declared = &parameter.declared;
            let Some(written) = &declared.type_ref else {
                let kind = TypeErrorKind::SignatureWithoutType(declared.name.text.clone());
                return Err(TypeError::at(declared.name.span, kind));
            };
            let held = self.written(resolved, written)?;
            boundary::crosses(&held, Crossing::Taken, &self.foreign, written.span)?;
            boundary::narrowed(parameter, &held)?;
            parameters.push(held);
        }
        let result = self.written(resolved, &declaration.result)?;
        let given_back = declaration.result.span;
        boundary::crosses(
            &result,
            Crossing::of(declaration),
            &self.foreign,
            given_back,
        )?;
        boundary::reaches_a_class(declaration, &parameters, &result, &self.foreign)?;
        boundary::builds_a_class(declaration, &result, &self.foreign)?;
        boundary::widens(declaration, &result)?;
        boundary::narrows(declaration, &result)?;
        boundary::stated_by(declaration)?;
        let signature = Type::function(parameters, result);
        self.bind(
            self.declared_at(&declaration.name),
            Scheme::monomorphic(signature),
        );
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
        self.bind(self.declared_at(&function.name), scheme);
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
            for constraint in &parameter.constraints {
                required.push(Required {
                    trait_name: constraint.name.text.clone(),
                    at: self.written(resolved, &constraint.argument)?,
                });
            }
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
                key: self.declared_at(&variant.name),
                labels: Vec::new(),
                carries: Vec::new(),
            }),
            VariantPayload::Tuple(written) => Ok(Built {
                key: self.declared_at(&variant.name),
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
            key: self.declared_at(name),
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
        let TypeRefKind::Named { path, arguments } = &written.kind else {
            return Ok(Type::Unit);
        };
        if let Some(module) = &path.module {
            let reached = Reached { module, path };
            return self.reached(&reached, self.each(resolved, arguments)?);
        }
        let name = &path.name;
        let definition = resolved
            .definition(Namespace::Type, name)
            .expect("name resolution gave every written type a definition");
        if definition.kind == DefinitionKind::TypeParameter {
            Self::counted(name, 0, arguments.len())?;
            return Ok(Type::Parameter(parameter_of(definition, name)));
        }
        let key = self.key_of(definition, name);
        Self::counted(name, *self.arities.get(&key).unwrap_or(&0), arguments.len())?;
        Ok(Type::Named {
            name: name.text.clone(),
            arguments: self.each(resolved, arguments)?,
        })
    }

    /// A type another module declares, reached through the name that module is imported by.
    ///
    /// # Errors
    ///
    /// Returns the name where that module declares no type of it, and the count where it does
    /// and the use writes the wrong number of arguments.
    fn reached(&self, reached: &Reached<'_>, arguments: Vec<Type>) -> Result<Type, TypeError> {
        let written = reached.path.written();
        let Some(takes) = self.arities.get(&Key::reached(&written.text)) else {
            let kind = TypeErrorKind::NotInModule {
                module: reached.module.text.clone(),
                name: reached.path.name.text.clone(),
            };
            return Err(TypeError::at(written.span, kind));
        };
        Self::counted(&written, *takes, arguments.len())?;
        Ok(Type::Named {
            name: written.text,
            arguments,
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

/// The type an instance is for, written over the arguments it writes it with.
///
/// `instance<T: Eq<T>> Eq<List<T>>` is for `List<T>`, and its methods are held to the trait's
/// signature at that type: `is_equal` takes two `List<T>` and gives back a `Bool`.
fn standing_for(declaration: &InstanceDeclaration) -> Vec<Type> {
    declares_each(declaration)
        .map(|parameter| Type::Parameter(TypeParameter::written(&parameter.name)))
        .collect()
}

/// What the instance asks of each argument its type is written with, in the order it writes them.
fn asked_of(declaration: &InstanceDeclaration) -> Vec<Vec<String>> {
    declares_each(declaration)
        .map(|parameter| {
            parameter
                .constraints
                .iter()
                .map(|constraint| constraint.name.text.clone())
                .collect()
        })
        .collect()
}

/// The type parameter the instance declares for each argument it writes its type with.
///
/// Name resolution has already refused an argument that is no parameter of the instance, which
/// `docs/specs/traits.md` states, so every argument names one of them.
fn declares_each(
    declaration: &InstanceDeclaration,
) -> impl Iterator<Item = &lumen_ast::TypeParameter> {
    declaration.arguments.iter().filter_map(|written| {
        declaration
            .type_parameters
            .iter()
            .find(|parameter| parameter.name.text == written.text)
    })
}

/// A written type reached through a module, which is the module written and the whole name.
struct Reached<'w> {
    module: &'w Name,
    path: &'w Path,
}
