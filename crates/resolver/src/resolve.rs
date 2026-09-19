//! The walk: every name of a module, pointed at the definition it means.

use std::collections::HashMap;

use lumen_ast::{Block, Expr, ExprKind, ForHeader, ForLoop, Function, IfExpr, Item, MatchExpr};
use lumen_ast::{Name, Pattern, PatternKind, Program, RecordField, Span, Statement, StatementKind};
use lumen_ast::{TypeDeclaration, TypeDefinition, TypeRef, TypeRefKind, Variant, VariantPayload};

use crate::definition::{Definition, DefinitionKind, Namespace, Origin};
use crate::error::{ResolveError, ResolveErrorKind};
use crate::prelude;
use crate::scope::Scope;

/// Resolves every name of `program`, or reports the first one that has no definition.
///
/// `docs/specs/modules.md` states the scopes this walks and the errors it raises.
///
/// # Errors
///
/// Returns the first name that names nothing, is declared twice, or hides something in scope.
pub fn resolve(program: Program) -> Result<ResolvedProgram, ResolveError> {
    let mut resolver = Resolver::new();
    resolver.module(&program)?;
    Ok(ResolvedProgram {
        program,
        definitions: resolver.definitions,
    })
}

/// A program whose every name points at the definition it means.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedProgram {
    program: Program,
    definitions: Definitions,
}

impl ResolvedProgram {
    /// The tree this was resolved from.
    #[must_use]
    pub const fn program(&self) -> &Program {
        &self.program
    }

    /// What `name` means where it is written, which is in one namespace or the other.
    ///
    /// A field label and a name reached through a `.` have none, because neither is a name in
    /// scope; `docs/specs/modules.md` says why.
    #[must_use]
    pub fn definition(&self, namespace: Namespace, name: &Name) -> Option<Definition> {
        self.definitions.get(&(namespace, name.span)).copied()
    }
}

/// What resolving one thing amounts to: it worked, or it is the one error of the run.
type Resolved = Result<(), ResolveError>;

/// The two scopes and what has been resolved into them so far.
struct Resolver {
    types: Scope,
    values: Scope,
    definitions: Definitions,
}

/// Every name that has a definition, keyed by the namespace it was written in and where.
type Definitions = HashMap<(Namespace, Span), Definition>;

impl Resolver {
    fn new() -> Self {
        Self {
            types: Scope::of_prelude(Namespace::Type, &prelude::TYPES, DefinitionKind::Type),
            values: Scope::of_prelude(
                Namespace::Value,
                &prelude::CONSTRUCTORS,
                DefinitionKind::Constructor,
            ),
            definitions: HashMap::new(),
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
            Item::Function(function) => {
                self.introduce_value(&function.name, DefinitionKind::Function)
            }
        }
    }

    /// A type declaration names a type, and names the value each of its variants is built with.
    fn declare_type(&mut self, declaration: &TypeDeclaration) -> Resolved {
        self.introduce_type(&declaration.name, DefinitionKind::Type)?;
        let TypeDefinition::Variants(variants) = &declaration.definition else {
            return self.introduce_value(&declaration.name, DefinitionKind::Constructor);
        };
        for variant in variants {
            self.introduce_value(&variant.name, DefinitionKind::Constructor)?;
        }
        Ok(())
    }

    fn define(&mut self, item: &Item) -> Resolved {
        match item {
            Item::Import(_) => Ok(()),
            Item::Type(declaration) => self.type_declaration(declaration),
            Item::Function(function) => self.function(function),
        }
    }

    fn type_declaration(&mut self, declaration: &TypeDeclaration) -> Resolved {
        self.types.enter();
        for parameter in &declaration.parameters {
            self.introduce_type(parameter, DefinitionKind::TypeParameter)?;
        }
        match &declaration.definition {
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
            self.introduce_type(parameter, DefinitionKind::TypeParameter)?;
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
            StatementKind::Binding { name, value, .. } => {
                self.expr(value)?;
                self.introduce_value(name, DefinitionKind::Local)
            }
            StatementKind::Assign { target, value, .. } => {
                self.expr(target)?;
                self.expr(value)
            }
            StatementKind::Return(value) => match value {
                Some(value) => self.expr(value),
                None => Ok(()),
            },
            StatementKind::Break | StatementKind::Continue => Ok(()),
            StatementKind::For(loop_) => self.for_loop(loop_),
            StatementKind::Expr(expr) => self.expr(expr),
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
            ExprKind::Name(name) => self.use_value(name),
            ExprKind::Integer(_) | ExprKind::String(_) | ExprKind::Bool(_) | ExprKind::Unit => {
                Ok(())
            }
            ExprKind::Unary { operand, .. } => self.expr(operand),
            ExprKind::Binary { left, right, .. } => {
                self.expr(left)?;
                self.expr(right)
            }
            ExprKind::Call { callee, arguments } => {
                self.expr(callee)?;
                self.each(arguments)
            }
            ExprKind::Field { receiver, .. } => self.expr(receiver),
            ExprKind::Try(inner) => self.expr(inner),
            ExprKind::Record { base, fields } => {
                self.use_value(base)?;
                for field in fields {
                    self.expr(&field.value)?;
                }
                Ok(())
            }
            ExprKind::If(chain) => self.if_expr(chain),
            ExprKind::Match(match_expr) => self.match_expr(match_expr),
        }
    }

    fn each(&mut self, exprs: &[Expr]) -> Resolved {
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

    fn pattern(&mut self, pattern: &Pattern) -> Resolved {
        match &pattern.kind {
            PatternKind::Name(name) => self.bare_pattern(name),
            PatternKind::Tuple { name, elements } => {
                self.use_value(name)?;
                for element in elements {
                    self.pattern(element)?;
                }
                Ok(())
            }
            PatternKind::Record { name, fields } => {
                self.use_value(name)?;
                for field in fields {
                    self.introduce_value(field, DefinitionKind::Local)?;
                }
                Ok(())
            }
            PatternKind::Integer(_) | PatternKind::String(_) | PatternKind::Bool(_) => Ok(()),
        }
    }

    /// A bare name matches what a constructor of that name carries, and otherwise binds the value.
    fn bare_pattern(&mut self, name: &Name) -> Resolved {
        if self.values.look_up(&name.text).is_some_and(is_constructor) {
            return self.use_value(name);
        }
        self.introduce_value(name, DefinitionKind::Local)
    }

    fn use_value(&mut self, name: &Name) -> Resolved {
        use_name(&self.values, &mut self.definitions, name)
    }

    fn type_ref(&mut self, type_ref: &TypeRef) -> Resolved {
        let TypeRefKind::Named { name, arguments } = &type_ref.kind else {
            return Ok(());
        };
        self.use_type(name)?;
        for argument in arguments {
            self.type_ref(argument)?;
        }
        Ok(())
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

/// What a name in `scope` means, recorded against where it is written.
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

fn is_constructor(definition: Definition) -> bool {
    definition.kind == DefinitionKind::Constructor
}
