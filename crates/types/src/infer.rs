//! The walk: every expression of a module given the type it has.

mod pattern;
mod record;
mod settle;

use std::collections::HashMap;
use std::mem;

use lumen_ast::{AssignOperator, BinaryOperator, Block, Expr, ExprKind, ForHeader};
use lumen_ast::{ForLoop, Function, IfExpr, Item, MatchExpr, Mutability, Name};
use lumen_ast::{Span, Statement, StatementKind, UnaryOperator};
use lumen_resolver::{Definition, DefinitionKind, Namespace, ResolvedProgram};

use crate::environment::{Environment, Key};
use crate::error::{Count, TypeError, TypeErrorKind};
use crate::infer::settle::Lookup;
use crate::scheme::{Quantified, Scheme};
use crate::table::Table;
use crate::types::{Type, TypeVar};
use crate::unify::{Clash, unify};

/// The type of every expression of `resolved`, or the first thing that has no type.
///
/// # Errors
///
/// Returns the first expression whose type inference cannot give it.
pub(crate) fn infer(resolved: &ResolvedProgram) -> Result<HashMap<Span, Type>, TypeError> {
    let mut table = Table::default();
    let environment = Environment::of(resolved, &mut table)?;
    let mut inference = Inference {
        resolved,
        environment,
        table,
        types: HashMap::new(),
        result: Type::Unit,
        introduced: Vec::new(),
        additions: Vec::new(),
        equalities: Vec::new(),
        lookups: Vec::new(),
    };
    inference.module()?;
    Ok(inference.solved())
}

/// Everything one run of inference is holding while it walks.
struct Inference<'a> {
    resolved: &'a ResolvedProgram,
    environment: Environment,
    table: Table,
    types: HashMap<Span, Type>,
    /// The result type of the function being walked, which `return` and `?` both answer to.
    result: Type,
    /// The names the current function bound, which go out of scope when it is done.
    introduced: Vec<Key>,
    /// The additions of the current function, which are `Int` unless something says otherwise.
    additions: Vec<(Type, Span)>,
    /// The comparisons of the current function, each waiting to be of a type that has `Eq`.
    equalities: Vec<(Type, Span)>,
    /// The fields of the current function, each waiting on the type it is reached through.
    lookups: Vec<Lookup>,
}

impl Inference<'_> {
    /// Each function bottom up, so a call reads a signature already inferred.
    ///
    /// A module is written top down: a definition sits below what uses it, which `docs/design.md`
    /// section 13 requires. Walking the other way is therefore walking uses last, and a function
    /// that declares no signature has been given one by the time anything calls it.
    fn module(&mut self) -> Result<(), TypeError> {
        let resolved = self.resolved;
        for item in resolved.program().items.iter().rev() {
            if let Item::Function(function) = item {
                self.function(function)?;
            }
        }
        Ok(())
    }

    fn function(&mut self, function: &Function) -> Result<(), TypeError> {
        let key = Key::at(&function.name);
        let scheme = self.scheme(&key);
        self.types.insert(function.name.span, scheme.body().clone());
        let Type::Function { parameters, result } = scheme.body().clone() else {
            unreachable!("a function is declared with a function type")
        };
        for (written, declared) in function.parameters.iter().zip(&parameters) {
            self.introduce(&written.name, Scheme::monomorphic(declared.clone()));
        }
        self.result = (*result).clone();
        let body = self.block(&function.body)?;
        self.look_up_fields()?;
        self.expect(&(*result).clone(), &body, function.body.span)?;
        self.settle_additions()?;
        self.settle_equalities()?;
        for gone in mem::take(&mut self.introduced) {
            self.environment.unbind(&gone);
        }
        self.generalise(key);
        Ok(())
    }

    fn block(&mut self, block: &Block) -> Result<Type, TypeError> {
        let mut value = Type::Unit;
        for statement in &block.statements {
            value = self.statement(statement)?;
        }
        Ok(value)
    }

    /// What a statement leaves behind, which is the block's value when it is the last one.
    fn statement(&mut self, statement: &Statement) -> Result<Type, TypeError> {
        match &statement.kind {
            StatementKind::Binding {
                mutability,
                name,
                value,
            } => self.binding(*mutability, name, value),
            StatementKind::Assign {
                target,
                operator,
                value,
            } => self.assign(target, *operator, value),
            StatementKind::Return(value) => self.returned(value.as_ref(), statement.span),
            StatementKind::Break | StatementKind::Continue => Ok(self.table.fresh()),
            StatementKind::For(walked) => {
                self.for_loop(walked)?;
                Ok(Type::Unit)
            }
            StatementKind::Expr(expr) => self.expr(expr),
        }
    }

    fn binding(
        &mut self,
        mutability: Mutability,
        name: &Name,
        value: &Expr,
    ) -> Result<Type, TypeError> {
        let bound = self.expr(value)?;
        let scheme = match mutability {
            Mutability::Mutable => Scheme::monomorphic(bound),
            Mutability::Immutable => self.generalised(&bound),
        };
        self.introduce(name, scheme);
        Ok(Type::Unit)
    }

    fn assign(
        &mut self,
        target: &Name,
        operator: AssignOperator,
        value: &Expr,
    ) -> Result<Type, TypeError> {
        let assigned = self.value(target);
        let found = self.expr(value)?;
        self.expect(&assigned, &found, value.span)?;
        if operator == AssignOperator::Add {
            self.additions.push((assigned, target.span));
        }
        Ok(Type::Unit)
    }

    /// A `return` leaves the block, so what follows it may be any type at all.
    fn returned(&mut self, value: Option<&Expr>, at: Span) -> Result<Type, TypeError> {
        let found = match value {
            Some(expr) => self.expr(expr)?,
            None => Type::Unit,
        };
        let at = value.map_or(at, |expr| expr.span);
        self.expect(&self.result.clone(), &found, at)?;
        Ok(self.table.fresh())
    }

    /// A loop has no value, so what its body leaves behind is discarded.
    fn for_loop(&mut self, walked: &ForLoop) -> Result<(), TypeError> {
        match &walked.header {
            ForHeader::Forever => {}
            ForHeader::While(condition) => self.condition(condition)?,
            ForHeader::In { binding, iterable } => {
                let over = self.expr(iterable)?;
                let item = self.table.fresh();
                self.expect(&Type::list(item.clone()), &over, iterable.span)?;
                self.introduce(binding, Scheme::monomorphic(item));
            }
        }
        self.block(&walked.body)?;
        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        let found = self.written(expr)?;
        self.types.insert(expr.span, found.clone());
        Ok(found)
    }

    fn written(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match &expr.kind {
            ExprKind::Name(name) => Ok(self.value(name)),
            ExprKind::Integer(_) => Ok(Type::int()),
            ExprKind::String(_) => Ok(Type::string()),
            ExprKind::Bool(_) => Ok(Type::boolean()),
            ExprKind::Unit => Ok(Type::Unit),
            ExprKind::Unary { operator, operand } => self.unary(*operator, operand),
            ExprKind::Binary {
                operator,
                left,
                right,
            } => self.binary(&Binary {
                operator: *operator,
                left,
                right,
                at: expr.span,
            }),
            ExprKind::Call { callee, arguments } => self.call(callee, arguments, expr.span),
            ExprKind::Field { receiver, name } => self.field(receiver, name),
            ExprKind::Try(inner) => self.propagated(inner, expr.span),
            ExprKind::Record { base, fields } => self.record(base, fields, expr.span),
            ExprKind::If(chain) => self.if_expr(chain),
            ExprKind::Match(matched) => self.match_expr(matched),
        }
    }

    fn unary(&mut self, operator: UnaryOperator, operand: &Expr) -> Result<Type, TypeError> {
        let wanted = match operator {
            UnaryOperator::Not => Type::boolean(),
            UnaryOperator::Negate => Type::int(),
        };
        let found = self.expr(operand)?;
        self.expect(&wanted, &found, operand.span)?;
        Ok(wanted)
    }

    fn binary(&mut self, written: &Binary<'_>) -> Result<Type, TypeError> {
        let Binary {
            operator,
            left,
            right,
            at,
        } = *written;
        let found = self.expr(left)?;
        let other = self.expr(right)?;
        match operator {
            BinaryOperator::Or | BinaryOperator::And => {
                self.expect(&Type::boolean(), &found, left.span)?;
                self.expect(&Type::boolean(), &other, right.span)?;
                Ok(Type::boolean())
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                self.expect(&found, &other, right.span)?;
                self.equalities.push((found, at));
                Ok(Type::boolean())
            }
            BinaryOperator::Less
            | BinaryOperator::LessOrEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterOrEqual => {
                self.expect(&Type::int(), &found, left.span)?;
                self.expect(&Type::int(), &other, right.span)?;
                Ok(Type::boolean())
            }
            BinaryOperator::Add => {
                self.expect(&found, &other, right.span)?;
                self.additions.push((found.clone(), left.span));
                Ok(found)
            }
            BinaryOperator::Divide | BinaryOperator::Remainder => {
                self.expect(&Type::int(), &found, left.span)?;
                self.expect(&Type::int(), &other, right.span)?;
                if right.kind == ExprKind::Integer(0) {
                    return Err(TypeError::at(right.span, TypeErrorKind::DivisorIsZero));
                }
                Ok(Type::option(Type::int()))
            }
            _ => {
                self.expect(&Type::int(), &found, left.span)?;
                self.expect(&Type::int(), &other, right.span)?;
                Ok(Type::int())
            }
        }
    }

    fn call(&mut self, callee: &Expr, arguments: &[Expr], at: Span) -> Result<Type, TypeError> {
        let signature = self.expr(callee)?;
        let mut given = Vec::new();
        for argument in arguments {
            given.push(self.expr(argument)?);
        }
        let Type::Function { parameters, result } = self.table.shallow(&signature) else {
            return self.applied(&signature, given, at);
        };
        if parameters.len() != given.len() {
            return match name_of(callee) {
                Some(name) => Err(miscounted(name, parameters.len(), given.len())),
                None => self.applied(&signature, given, at),
            };
        }
        for ((wanted, found), argument) in parameters.iter().zip(&given).zip(arguments) {
            self.expect(wanted, found, argument.span)?;
        }
        Ok(*result)
    }

    /// A call of something whose type is not yet a function, which unification has to settle.
    fn applied(&mut self, called: &Type, given: Vec<Type>, at: Span) -> Result<Type, TypeError> {
        let result = self.table.fresh();
        self.expect(called, &Type::function(given, result.clone()), at)?;
        Ok(result)
    }

    /// A field is looked up as soon as the type it is reached through is known, and waits when it
    /// is not.
    ///
    /// Waiting costs the reader something: what the field turns out to be is then reported where
    /// the wait ended rather than where the field is written. So the wait is only for the types
    /// that something further on has still to settle.
    fn field(&mut self, receiver: &Expr, name: &Name) -> Result<Type, TypeError> {
        let through = self.expr(receiver)?;
        let found = self.table.fresh();
        let lookup = Lookup {
            through,
            field: name.clone(),
            found: found.clone(),
        };
        if matches!(self.table.shallow(&lookup.through), Type::Var(_)) {
            self.lookups.push(lookup);
            return Ok(found);
        }
        self.look_up(lookup)?;
        Ok(found)
    }

    fn propagated(&mut self, inner: &Expr, at: Span) -> Result<Type, TypeError> {
        let found = self.expr(inner)?;
        let ok = self.table.fresh();
        let error = self.table.fresh();
        self.expect(&Type::result(ok.clone(), error.clone()), &found, inner.span)?;
        let propagated = self.table.fresh();
        self.expect(&self.result.clone(), &Type::result(propagated, error), at)?;
        Ok(ok)
    }

    fn if_expr(&mut self, chain: &IfExpr) -> Result<Type, TypeError> {
        let mut blocks = Vec::new();
        for branch in &chain.branches {
            self.condition(&branch.condition)?;
            blocks.push((self.block(&branch.block)?, branch.block.span));
        }
        let result = match &chain.otherwise {
            Some(block) => self.block(block)?,
            None => Type::Unit,
        };
        for (found, span) in blocks {
            self.expect(&result, &found, span)?;
        }
        Ok(result)
    }

    fn match_expr(&mut self, matched: &MatchExpr) -> Result<Type, TypeError> {
        let scrutinee = self.expr(&matched.scrutinee)?;
        let result = self.table.fresh();
        for arm in &matched.arms {
            self.pattern(&arm.pattern, &scrutinee)?;
            let found = self.expr(&arm.body)?;
            self.expect(&result, &found, arm.body.span)?;
        }
        Ok(result)
    }
    fn condition(&mut self, condition: &Expr) -> Result<(), TypeError> {
        let found = self.expr(condition)?;
        self.expect(&Type::boolean(), &found, condition.span)
    }

    /// Makes `expected` and `found` one type, or says which of the two the source wrote.
    fn expect(&mut self, expected: &Type, found: &Type, at: Span) -> Result<(), TypeError> {
        match unify(&mut self.table, expected, found) {
            Ok(()) => Ok(()),
            Err(Clash::Infinite) => Err(TypeError::at(at, TypeErrorKind::Infinite)),
            Err(Clash::Mismatch) => {
                let kind = TypeErrorKind::Mismatch {
                    expected: self.table.solved(expected),
                    found: self.table.solved(found),
                };
                Err(TypeError::at(at, kind))
            }
        }
    }

    /// The type `name` has at this one use, fresh in whatever its definition is free over.
    fn value(&mut self, name: &Name) -> Type {
        let key = self.key_of(name);
        let scheme = self.scheme(&key);
        scheme.instantiate(&mut self.table)
    }

    /// Binds `name` to `scheme`, and records the type at the name, which is where it is declared.
    fn introduce(&mut self, name: &Name, scheme: Scheme) {
        self.types.insert(name.span, scheme.body().clone());
        let key = Key::at(name);
        self.environment.bind(key.clone(), scheme);
        self.introduced.push(key);
    }

    /// The scheme the name defined at `key` was given, which every phase before this one assures.
    fn scheme(&self, key: &Key) -> Scheme {
        self.environment
            .scheme(key)
            .cloned()
            .expect("name resolution gave every name a definition that was declared a type")
    }

    fn key_of(&self, name: &Name) -> Key {
        Key::of(self.definition(name), name)
    }

    fn definition_kind(&self, name: &Name) -> DefinitionKind {
        self.definition(name).kind
    }

    fn definition(&self, name: &Name) -> Definition {
        self.resolved
            .definition(Namespace::Value, name)
            .expect("name resolution gave every written value a definition")
    }

    /// Frees the type at `key` over whatever nothing else in scope holds.
    fn generalise(&mut self, key: Key) {
        let Some(shell) = self.environment.unbind(&key) else {
            return;
        };
        let body = self.table.solved(shell.body());
        let over = self.free_in(&body);
        let quantified = shell
            .quantified()
            .iter()
            .cloned()
            .chain(over.into_iter().map(Quantified::Var))
            .collect();
        self.environment.bind(key, Scheme::over(quantified, body));
    }

    /// A binding is as polymorphic as the value it was given, which is let-polymorphism.
    fn generalised(&mut self, bound: &Type) -> Scheme {
        let body = self.table.solved(bound);
        let over = self.free_in(&body);
        if over.is_empty() {
            return Scheme::monomorphic(body);
        }
        Scheme::over(over.into_iter().map(Quantified::Var).collect(), body)
    }

    /// The variables of `body` that nothing else holds, which are the ones free to quantify.
    ///
    /// A constraint still waiting holds its variables as firmly as a name in scope does: quantify
    /// one of those and every later use gets a copy the constraint will never reach.
    fn free_in(&self, body: &Type) -> Vec<TypeVar> {
        let mut free = Vec::new();
        self.table.unsettled(body, &mut free);
        if free.is_empty() {
            return free;
        }
        let mut held = Vec::new();
        for scheme in self.environment.schemes() {
            self.table.unsettled(scheme.body(), &mut held);
        }
        for lookup in &self.lookups {
            self.table.unsettled(&lookup.through, &mut held);
            self.table.unsettled(&lookup.found, &mut held);
        }
        for (added, _) in &self.additions {
            self.table.unsettled(added, &mut held);
        }
        for (compared, _) in &self.equalities {
            self.table.unsettled(compared, &mut held);
        }
        free.retain(|var| !held.contains(var));
        free
    }

    /// Every type this run recorded, each followed to what it was settled on.
    fn solved(&self) -> HashMap<Span, Type> {
        self.types
            .iter()
            .map(|(span, found)| (*span, self.table.solved(found)))
            .collect()
    }
}

/// A binary expression as inference reads it: the operator, its two operands, and its span.
#[derive(Clone, Copy)]
struct Binary<'a> {
    operator: BinaryOperator,
    left: &'a Expr,
    right: &'a Expr,
    /// The whole expression, which is what a refused comparison points the reader at.
    at: Span,
}

/// The index `field` is at, or the report that nothing of that name is there.
fn labelled(labels: &[String], field: &Name, of: &Type) -> Result<usize, TypeError> {
    labels
        .iter()
        .position(|label| *label == field.text)
        .ok_or_else(|| {
            let kind = TypeErrorKind::UnknownField {
                of: of.clone(),
                field: field.text.clone(),
            };
            TypeError::at(field.span, kind)
        })
}

fn miscounted(name: &Name, takes: usize, given: usize) -> TypeError {
    let count = Count {
        name: name.text.clone(),
        takes,
        given,
    };
    TypeError::at(name.span, TypeErrorKind::WrongArgumentCount(count))
}

/// The name a call is a call of, when the thing being called is written as one.
fn name_of(callee: &Expr) -> Option<&Name> {
    match &callee.kind {
        ExprKind::Name(name) | ExprKind::Field { name, .. } => Some(name),
        _ => None,
    }
}
