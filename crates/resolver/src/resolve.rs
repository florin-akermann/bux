//! The walk: every name of a module, pointed at the definition it means.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use lumen_ast::{Block, Expr, ExprKind, ExternDeclaration, ForHeader, ForLoop, Function};
use lumen_ast::{IfExpr, Item};
use lumen_ast::{MatchExpr, Mutability, Name, Path, Program, RecordField};
use lumen_ast::{Span, Statement, StatementKind, TypeDeclaration, TypeDefinition, TypeRef};
use lumen_ast::{TypeRefKind, Variant, VariantPayload};

use crate::definition::{Definition, DefinitionKind, Namespace, Origin};
use crate::error::{ResolveError, ResolveErrorKind};
use crate::library::PRELUDE;
use crate::order;
use crate::prelude;
use crate::scope::Scope;

mod pattern;
mod program;
mod traits;

pub use program::ResolvedProgram;

/// Resolves every name of `program`, or reports the first one that has no definition.
///
/// `docs/specs/modules.md` states the scopes this walks and the errors it raises.
///
/// # Errors
///
/// Returns the first name that names nothing, is declared twice, or hides something in scope,
/// and then the first declaration written above something that uses it.
pub fn resolve(program: Program, module: &str) -> Result<ResolvedProgram, ResolveError> {
    let mut resolver = Resolver::new();
    resolver.module(&program)?;
    let definitions = resolver.definitions;
    if let Some(error) = order::out_of_order(&program, &definitions) {
        return Err(error);
    }
    Ok(ResolvedProgram::new(
        module.to_owned(),
        program,
        definitions,
    ))
}

/// The prelude, resolved, which every other module's scope is seeded from.
///
/// `docs/specs/library.md` says the compiler carries the source; this is that source with every
/// name in it pointed at what it means, worked out once and the same for every module.
#[must_use]
pub fn prelude_resolved() -> &'static ResolvedProgram {
    static RESOLVED: LazyLock<ResolvedProgram> = LazyLock::new(|| {
        let program = prelude::program().clone();
        let mut resolver = Resolver::of_the_library();
        resolver
            .module(&program)
            .expect("every name of the library the compiler carries resolves");
        ResolvedProgram::new(PRELUDE.to_owned(), program, resolver.definitions)
    });
    &RESOLVED
}

/// What resolving one thing amounts to: it worked, or it is the one error of the run.
type Resolved = Result<(), ResolveError>;

/// The two scopes and what has been resolved into them so far.
struct Resolver {
    types: Scope,
    values: Scope,
    definitions: Definitions,
    /// Every trait that has an instance for a type, which is what a second one is refused by.
    instances: HashSet<(String, String)>,
    /// The methods each trait this module declares declares, which its instances must write.
    declared_traits: HashMap<String, Vec<String>>,
}

/// Every name that has a definition, keyed by the namespace it was written in and where.
type Definitions = HashMap<(Namespace, Span), Definition>;

impl Resolver {
    /// A resolver for a module of a program, whose scope is the prelude and nothing else.
    ///
    /// `docs/specs/library.md` says `push` and `at` are the compiler's rather than a source, and
    /// a module reaches them by importing `list`, so no program module has either name in scope.
    fn new() -> Self {
        Self {
            types: Scope::of_prelude(
                Namespace::Type,
                &[
                    (&prelude::types(), DefinitionKind::Type),
                    (&prelude::trait_names(), DefinitionKind::Trait),
                ],
            ),
            values: Scope::of_prelude(
                Namespace::Value,
                &[
                    (&prelude::constructors(), DefinitionKind::Constructor),
                    (&prelude::functions(), DefinitionKind::Function),
                    (&prelude::method_names(), DefinitionKind::TraitMethod),
                ],
            ),
            definitions: HashMap::new(),
            instances: traits::prelude_instances(),
            declared_traits: HashMap::new(),
        }
    }

    /// A resolver for the prelude itself, whose scope holds what the JVM holds and nothing else.
    fn of_the_library() -> Self {
        Self {
            types: Scope::of_prelude(Namespace::Type, &[(&prelude::held(), DefinitionKind::Type)]),
            values: Scope::of_prelude(
                Namespace::Value,
                &[
                    (&prelude::supplied(), DefinitionKind::Function),
                    (&prelude::held_for(PRELUDE), DefinitionKind::Function),
                ],
            ),
            definitions: HashMap::new(),
            instances: HashSet::new(),
            declared_traits: HashMap::new(),
        }
    }

    /// Every name a module declares is in scope before any body is walked.
    fn module(&mut self, program: &Program) -> Resolved {
        self.types.enter();
        self.values.enter();
        for item in &program.items {
            self.declare(item)?;
        }
        for item in &program.items {
            self.define(item)?;
        }
        Ok(())
    }

    fn declare(&mut self, item: &Item) -> Resolved {
        match item {
            Item::Import(import) => self.introduce_value(&import.module, DefinitionKind::Module),
            Item::Type(declaration) => self.declare_type(declaration),
            Item::Trait(declaration) => self.declare_trait(declaration),
            Item::Instance(_) | Item::Derive(_) => Ok(()),
            Item::Function(function) => {
                self.introduce_value(&function.name, DefinitionKind::Function)
            }
            Item::Extern(declaration) => {
                self.introduce_value(&declaration.name, DefinitionKind::Function)
            }
        }
    }

    /// A type declaration names a type, and names the value each of its variants is built with.
    ///
    /// A Java class names no value: `docs/specs/interop.md` builds one with an `extern new` and
    /// with nothing else, so its Lumen name is a type and never a constructor.
    fn declare_type(&mut self, declaration: &TypeDeclaration) -> Resolved {
        self.introduce_type(&declaration.name, DefinitionKind::Type)?;
        match &declaration.definition {
            TypeDefinition::Foreign { .. } => Ok(()),
            TypeDefinition::Record(_) => {
                self.introduce_value(&declaration.name, DefinitionKind::Constructor)
            }
            TypeDefinition::Variants(variants) => {
                for variant in variants {
                    self.introduce_value(&variant.name, DefinitionKind::Constructor)?;
                }
                Ok(())
            }
        }
    }

    fn define(&mut self, item: &Item) -> Resolved {
        match item {
            Item::Import(_) => Ok(()),
            Item::Type(declaration) => self.type_declaration(declaration),
            Item::Trait(declaration) => self.trait_declaration(declaration),
            Item::Instance(declaration) => self.instance(declaration),
            Item::Derive(declaration) => self.derive(declaration),
            Item::Function(function) => self.function(function),
            Item::Extern(declaration) => self.declared_extern(declaration),
        }
    }

    /// An `extern` names the types of its signature, and its parameters while it names them.
    ///
    /// There is no body below it, so nothing those names are in scope for; they are introduced
    /// all the same, because two parameters of one name is the mistake it always was.
    fn declared_extern(&mut self, declaration: &ExternDeclaration) -> Resolved {
        self.values.enter();
        for parameter in &declaration.parameters {
            if let Some(type_ref) = &parameter.declared.type_ref {
                self.type_ref(type_ref)?;
            }
            self.introduce_value(&parameter.declared.name, DefinitionKind::Parameter)?;
        }
        self.type_ref(&declaration.result)?;
        self.values.leave();
        Ok(())
    }

    fn type_declaration(&mut self, declaration: &TypeDeclaration) -> Resolved {
        self.types.enter();
        for parameter in &declaration.parameters {
            self.introduce_type(parameter, DefinitionKind::TypeParameter)?;
        }
        match &declaration.definition {
            // A Java class names no Lumen type below its own, so there is nothing to resolve.
            TypeDefinition::Foreign { .. } => {}
            TypeDefinition::Record(fields) => self.record_fields(fields)?,
            TypeDefinition::Variants(variants) => {
                for variant in variants {
                    self.variant(variant)?;
                }
            }
        }
        self.types.leave();
        Ok(())
    }

    fn variant(&mut self, variant: &Variant) -> Resolved {
        match &variant.payload {
            VariantPayload::None => Ok(()),
            VariantPayload::Tuple(types) => {
                for type_ref in types {
                    self.type_ref(type_ref)?;
                }
                Ok(())
            }
            VariantPayload::Record(fields) => self.record_fields(fields),
        }
    }

    fn record_fields(&mut self, fields: &[RecordField]) -> Resolved {
        for field in fields {
            self.type_ref(&field.type_ref)?;
        }
        Ok(())
    }

    fn function(&mut self, function: &Function) -> Resolved {
        self.types.enter();
        self.values.enter();
        for parameter in &function.type_parameters {
            self.introduce_type(&parameter.name, DefinitionKind::TypeParameter)?;
        }
        for parameter in &function.type_parameters {
            self.constrained(parameter)?;
        }
        for parameter in &function.parameters {
            if let Some(type_ref) = &parameter.type_ref {
                self.type_ref(type_ref)?;
            }
            self.introduce_value(&parameter.name, DefinitionKind::Parameter)?;
        }
        if let Some(result) = &function.result {
            self.type_ref(result)?;
        }
        self.block(&function.body)?;
        self.values.leave();
        self.types.leave();
        Ok(())
    }

    fn block(&mut self, block: &Block) -> Resolved {
        self.values.enter();
        for statement in &block.statements {
            self.statement(statement)?;
        }
        self.values.leave();
        Ok(())
    }

    fn statement(&mut self, statement: &Statement) -> Resolved {
        match &statement.kind {
            StatementKind::Binding {
                mutability,
                name,
                value,
            } => {
                self.expr(value)?;
                self.introduce_value(name, bound(*mutability))
            }
            StatementKind::Assign { target, value, .. } => {
                self.assigned_to(target)?;
                self.expr(value)
            }
            StatementKind::Return(value) => match value {
                Some(value) => self.expr(value),
                None => Ok(()),
            },
            StatementKind::Break | StatementKind::Continue => Ok(()),
            StatementKind::For(loop_) => self.for_loop(loop_),
            StatementKind::Discard(expr) | StatementKind::Expr(expr) => self.expr(expr),
        }
    }

    fn for_loop(&mut self, loop_: &ForLoop) -> Resolved {
        self.values.enter();
        match &loop_.header {
            ForHeader::Forever => {}
            ForHeader::While(condition) => self.expr(condition)?,
            ForHeader::In { binding, iterable } => {
                self.expr(iterable)?;
                self.introduce_value(binding, DefinitionKind::Local)?;
            }
        }
        self.block(&loop_.body)?;
        self.values.leave();
        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> Resolved {
        match &expr.kind {
            ExprKind::Name(name) => self.named(name, Position::Value),
            ExprKind::Integer(_) | ExprKind::String(_) | ExprKind::Bool(_) | ExprKind::Unit => {
                Ok(())
            }
            ExprKind::Unary { operand, .. } => self.expr(operand),
            ExprKind::Binary { left, right, .. } => {
                self.expr(left)?;
                self.expr(right)
            }
            ExprKind::Call { callee, arguments } => {
                self.written(callee, Position::Callee)?;
                self.each(&arguments.values())
            }
            ExprKind::Field { receiver, .. } => self.written(receiver, Position::Receiver),
            ExprKind::Try(inner) => self.expr(inner),
            ExprKind::List(elements) => {
                for element in elements {
                    self.expr(element)?;
                }
                Ok(())
            }
            ExprKind::Record { base, fields } => {
                self.built_as(base)?;
                for field in fields {
                    self.expr(&field.value)?;
                }
                Ok(())
            }
            ExprKind::If(chain) => self.if_expr(chain),
            ExprKind::Match(match_expr) => self.match_expr(match_expr),
        }
    }

    /// The name an assignment changes, which is a `var` binding and nothing else.
    ///
    /// Mutation is explicit, so what a name holds changes only where `var` said it may. What
    /// this adds beyond that is the name that binds nothing at all: a constructor names a way
    /// to build a value, and lowering it would store into a slot that does not exist.
    fn assigned_to(&mut self, name: &Name) -> Resolved {
        self.named(name, Position::Value)?;
        match self.values.look_up(&name.text) {
            Some(found) if found.kind == DefinitionKind::Variable => Ok(()),
            _ => Err(refused(name, ResolveErrorKind::NotAVariable)),
        }
    }

    /// An expression in a position that takes more than a value, when it is a bare name.
    fn written(&mut self, expr: &Expr, position: Position) -> Resolved {
        match &expr.kind {
            ExprKind::Name(name) => self.named(name, position),
            ExprKind::Field { receiver, name } if matches!(position, Position::Callee) => {
                self.calls(receiver, name)
            }
            _ => self.expr(expr),
        }
    }

    /// The name a call writes after a `.`, which the name before the dot says the meaning of.
    ///
    /// `io.println(…)` reaches a function of the module `io`, and only that module says what is
    /// there. `maybe.or(0)` is the call `or(maybe, 0)`, which `docs/specs/calls.md` states, so
    /// the name after the dot is a name of this module and is looked up here like any other.
    fn calls(&mut self, receiver: &Expr, name: &Name) -> Resolved {
        self.written(receiver, Position::Receiver)?;
        if self.reaches_a_module(receiver) {
            return Ok(());
        }
        self.named(name, Position::Callee)
    }

    /// Whether the name before the dot is a module, which is what a name is reached through.
    ///
    /// The scope is what says so here, because the resolution that would answer it is the one
    /// being built; [`ResolvedProgram::module_reached`] is the same question asked afterwards.
    fn reaches_a_module(&self, receiver: &Expr) -> bool {
        let ExprKind::Name(module) = &receiver.kind else {
            return false;
        };
        self.values
            .look_up(&module.text)
            .is_some_and(|found| found.kind == DefinitionKind::Module)
    }

    /// Resolves `name`, then refuses it when what it names is no value and the position wants one.
    ///
    /// Version 0.1 has no type and no shape for a function or a module held as a value, so there
    /// is nothing for lowering to write. The name says what it is here, so here is where it goes.
    fn named(&mut self, name: &Name, position: Position) -> Resolved {
        self.use_value(name)?;
        let Some(found) = self.values.look_up(&name.text) else {
            return Ok(());
        };
        if position.takes(found.kind) {
            return Ok(());
        }
        match found.kind {
            DefinitionKind::Function | DefinitionKind::TraitMethod => {
                Err(refused(name, ResolveErrorKind::NotCalled))
            }
            DefinitionKind::Module => Err(refused(name, ResolveErrorKind::NotReachedThrough)),
            _ => Ok(()),
        }
    }

    fn each(&mut self, exprs: &[&Expr]) -> Resolved {
        for expr in exprs {
            self.expr(expr)?;
        }
        Ok(())
    }

    fn if_expr(&mut self, chain: &IfExpr) -> Resolved {
        for branch in &chain.branches {
            self.expr(&branch.condition)?;
            self.block(&branch.block)?;
        }
        match &chain.otherwise {
            Some(block) => self.block(block),
            None => Ok(()),
        }
    }

    fn match_expr(&mut self, match_expr: &MatchExpr) -> Resolved {
        self.expr(&match_expr.scrutinee)?;
        for arm in &match_expr.arms {
            self.values.enter();
            self.pattern(&arm.pattern)?;
            self.expr(&arm.body)?;
            self.values.leave();
        }
        Ok(())
    }

    fn type_ref(&mut self, type_ref: &TypeRef) -> Resolved {
        let TypeRefKind::Named { path, arguments } = &type_ref.kind else {
            return Ok(());
        };
        if path.module.is_some() {
            self.reached_through(path)?;
        } else {
            self.named_type(&path.name)?;
        }
        for argument in arguments {
            self.type_ref(argument)?;
        }
        Ok(())
    }

    /// The module a path is reached through, which is a name in scope and has to be a module.
    ///
    /// Nothing after the dot is resolved here: what that module declares is what answers it, and
    /// `docs/specs/modules.md` leaves that to the phase that holds the other module's surface.
    fn reached_through(&mut self, path: &Path) -> Resolved {
        let Some(module) = &path.module else {
            unreachable!("a path with no module is resolved as the name of this module it is")
        };
        self.use_value(module)?;
        match self.values.look_up(&module.text) {
            Some(found) if found.kind == DefinitionKind::Module => Ok(()),
            _ => Err(refused(module, ResolveErrorKind::NotAModule)),
        }
    }

    fn use_value(&mut self, name: &Name) -> Resolved {
        use_name(&self.values, &mut self.definitions, name)
    }

    /// A type as a type is written, refusing a trait, which names what a type can do and is none.
    fn named_type(&mut self, name: &Name) -> Resolved {
        self.use_type(name)?;
        if self.types.look_up(&name.text).map(|one| one.kind) != Some(DefinitionKind::Trait) {
            return Ok(());
        }
        let kind = ResolveErrorKind::TraitAsType(name.text.clone());
        Err(ResolveError::at(name, kind))
    }

    fn use_type(&mut self, name: &Name) -> Resolved {
        use_name(&self.types, &mut self.definitions, name)
    }

    fn introduce_value(&mut self, name: &Name, kind: DefinitionKind) -> Resolved {
        introduce(&mut self.values, &mut self.definitions, name, kind)
    }

    fn introduce_type(&mut self, name: &Name, kind: DefinitionKind) -> Resolved {
        introduce(&mut self.types, &mut self.definitions, name, kind)
    }
}

/// The kind of definition a binding makes, which is what decides whether it may change.
const fn bound(mutability: Mutability) -> DefinitionKind {
    match mutability {
        Mutability::Immutable => DefinitionKind::Local,
        Mutability::Mutable => DefinitionKind::Variable,
    }
}

/// Where a name is written, which is what decides whether it must name a value.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Position {
    /// Anywhere a value belongs, which is neither a function nor a module.
    Value,
    /// The name of a call, which is the one place a function name is written.
    Callee,
    /// The left of a `.`, which is the one place a module name is written.
    Receiver,
}

impl Position {
    /// Whether a name defined as `kind` belongs here beyond an ordinary value.
    const fn takes(self, kind: DefinitionKind) -> bool {
        match self {
            Self::Value => false,
            Self::Callee => matches!(kind, DefinitionKind::Function | DefinitionKind::TraitMethod),
            Self::Receiver => matches!(kind, DefinitionKind::Module),
        }
    }
}

/// The failure `name` is refused with, worded by what the name is.
fn refused(name: &Name, worded: fn(String) -> ResolveErrorKind) -> ResolveError {
    ResolveError::at(name, worded(name.text.clone()))
}

/// What a name in `scope` means, recorded against the namespace and the place it is written.
fn use_name(scope: &Scope, found: &mut Definitions, name: &Name) -> Resolved {
    let Some(definition) = scope.look_up(&name.text) else {
        let kind = ResolveErrorKind::missing(scope.namespace(), &name.text);
        return Err(ResolveError::at(name, kind));
    };
    found.insert((scope.namespace(), name.span), definition);
    Ok(())
}

/// Declares `name` in `scope`, and points the name at the definition it has just made.
fn introduce(
    scope: &mut Scope,
    found: &mut Definitions,
    name: &Name,
    kind: DefinitionKind,
) -> Resolved {
    let namespace = scope.namespace();
    scope
        .introduce(name, kind)
        .map_err(|clash| ResolveError::at(name, ResolveErrorKind::of(clash, &name.text)))?;
    found.insert(
        (namespace, name.span),
        Definition {
            kind,
            origin: Origin::Declared(name.span),
        },
    );
    Ok(())
}
