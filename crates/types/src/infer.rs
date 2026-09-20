//! The walk: every expression of a module given the type it has.

mod arguments;
mod flags;
mod operator;
mod pattern;
mod predicate;
mod record;
mod settle;

use std::collections::HashMap;
use std::mem;

use lumen_ast::ForHeader;
use lumen_ast::{Arguments, AssignOperator, BinaryOperator, Block, Expr, ExprKind};
use lumen_ast::{ForLoop, Function, IfExpr, Item, MatchExpr, Mutability, Name};
use lumen_ast::{Span, Statement, StatementKind};
use lumen_resolver::{Definition, DefinitionKind, Namespace, ResolvedProgram};

use crate::environment::{Environment, Key};
use crate::error::{Count, TypeError, TypeErrorKind};
use crate::infer::operator::Operated;
use crate::infer::settle::Lookup;
use crate::scheme::{Quantified, Required, Scheme};
use crate::surface::{Imported, Surface};
use crate::table::Table;
use crate::types::{OPTION, RESULT, Type, TypeVar};
use crate::unify::{Clash, unify};

/// What one run of inference answers: every type it settled, and everything read off them.
pub(crate) struct Inferred {
    /// The type of whatever is written at each span, which every expression and name has.
    pub(crate) types: HashMap<Span, Type>,
    /// What the module offers a module that imports it.
    pub(crate) surface: Surface,
    /// The type each use of a trait method reached its instance at, and nothing else has one.
    pub(crate) methods: HashMap<Span, Type>,
}

/// The type of every expression of `resolved`, what the module offers, and the type each use of
/// a trait method reached its instance at — or the first thing that has no type.
///
/// # Errors
///
/// Returns the first expression whose type inference cannot give it.
pub(crate) fn infer(
    resolved: &ResolvedProgram,
    imported: &Imported,
) -> Result<Inferred, TypeError> {
    let mut table = Table::default();
    let environment = Environment::of(resolved, &mut table)?;
    let mut inference = Inference {
        resolved,
        imported,
        environment,
        table,
        types: HashMap::new(),
        result: Type::Unit,
        introduced: Vec::new(),
        operated: Vec::new(),
        discards: Vec::new(),
        lookups: Vec::new(),
        propagations: Vec::new(),
        requirements: Vec::new(),
        promised: Vec::new(),
        methods_at: HashMap::new(),
    };
    inference.module()?;
    let surface = Surface::of(resolved, inference.offered());
    let methods = inference.methods_at.clone();
    Ok(Inferred {
        types: inference.solved(),
        surface,
        methods,
    })
}

/// Everything one run of inference is holding while it walks.
struct Inference<'a> {
    resolved: &'a ResolvedProgram,
    /// What the modules this one imports offer it, which is how a name inside one has a type.
    imported: &'a Imported,
    environment: Environment,
    table: Table,
    types: HashMap<Span, Type>,
    /// The result type of the function being walked, which `return` and `?` both answer to.
    result: Type,
    /// The names the current function bound, which go out of scope when it is done.
    introduced: Vec<Key>,
    /// The operators of the current function, each `Int` unless something says otherwise.
    operated: Vec<Operated>,
    /// The statements of the current function that nothing takes the value of, each of which
    /// must therefore have no value to take.
    discards: Vec<(Type, Span)>,
    /// The fields of the current function, each waiting on the type it is reached through.
    lookups: Vec<Lookup>,
    /// The `?`s of the current function that nothing had yet said which kind they propagate.
    propagations: Vec<Propagation>,
    /// The traits the current function must answer for, each at the type it was asked of.
    requirements: Vec<Requirement>,
    /// The traits the current function's own type parameters promise, which answer for it.
    promised: Vec<Required>,
    /// The type each use of a trait method reached its instance at, by where the use is written.
    methods_at: HashMap<Span, Type>,
}

impl Inference<'_> {
    /// Each function bottom up, so a call reads a signature already inferred.
    ///
    /// A module is written top down: a definition sits below what uses it, which `docs/design.md`
    /// section 13 requires. Walking the other way is therefore walking uses last, and a function
    /// that declares no signature has been given one by the time anything calls it.
    fn module(&mut self) -> Result<(), TypeError> {
        let resolved = self.resolved;
        for item in &resolved.program().items {
            let Item::Trait(declaration) = item else {
                continue;
            };
            for method in &declaration.methods {
                self.settle_method(method)?;
            }
        }
        let written: Vec<&Function> = resolved.program().functions().collect();
        for function in written.into_iter().rev() {
            self.function(function)?;
        }
        Ok(())
    }

    /// The type of every function this module declares, which is what it offers.
    ///
    /// Inference walks bottom up and generalises each function as it leaves it, so by here
    /// every one of them carries the type a module importing it would read.
    fn offered(&self) -> HashMap<String, Scheme> {
        self.resolved
            .program()
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(function) => {
                    let scheme = self
                        .environment
                        .scheme(&Key::at(&function.name))
                        .expect("every function a module declares is bound before its body");
                    Some((function.name.text.clone(), scheme.clone()))
                }
                Item::Import(_) | Item::Type(_) | Item::Trait(_) | Item::Instance(_) => None,
            })
            .collect()
    }

    fn function(&mut self, function: &Function) -> Result<(), TypeError> {
        let key = Key::at(&function.name);
        let scheme = self.scheme(&key);
        self.promised = scheme.required().to_vec();
        self.types.insert(function.name.span, scheme.body().clone());
        self.writes_the_method(&key, scheme.body(), function.name.span)?;
        let Type::Function { parameters, result } = scheme.body().clone() else {
            unreachable!("a function is declared with a function type")
        };
        for (written, declared) in function.parameters.iter().zip(&parameters) {
            self.introduce(&written.name, Scheme::monomorphic(declared.clone()));
        }
        self.result = (*result).clone();
        let body = self.block(&function.body, Gives::ItsValue)?;
        self.look_up_fields()?;
        self.expect(&(*result).clone(), &body, function.body.span)?;
        self.settle_propagations()?;
        self.settle_operators()?;
        self.settle_requirements()?;
        self.settle_discards()?;
        self.settle_parameters(function)?;
        self.settle_predicate(function)?;
        for gone in mem::take(&mut self.introduced) {
            self.environment.unbind(&gone);
        }
        self.generalise(key);
        Ok(())
    }

    /// Holds an instance's method to the signature its trait gives it at the instance's type.
    ///
    /// A body may leave a type out and let this supply it, which is why the two are met rather
    /// than compared: `fn equals(a, b)` in `instance Eq<Point>` takes two `Point`s because the
    /// trait says so.
    fn writes_the_method(&mut self, key: &Key, found: &Type, at: Span) -> Result<(), TypeError> {
        let Some(declared) = self.environment.written_as(key).cloned() else {
            return Ok(());
        };
        self.expect(&declared, &found.clone(), at)
    }

    /// What a block leaves behind, having refused every statement that leaves something unread.
    ///
    /// Every statement but the last is written for its effect, and the last one too when the
    /// block gives its value to nothing; `docs/specs/discarding.md` states the rule.
    fn block(&mut self, block: &Block, gives: Gives) -> Result<Type, TypeError> {
        let mut value = Type::Unit;
        let last = block.statements.len().saturating_sub(1);
        for (at, statement) in block.statements.iter().enumerate() {
            value = self.statement(statement)?;
            if at < last || gives == Gives::Nothing {
                self.discarded(statement, &value);
            }
        }
        Ok(value)
    }

    /// Records a statement whose value nothing takes, to be answered once the function is done.
    ///
    /// Only a bare expression is asked about. A binding, an assignment, and a `for` leave `()`,
    /// and `_ =` says the value was thrown away on purpose; `return`, `break`, and `continue`
    /// leave a fresh variable because control has already gone, and asking `()` of that would
    /// settle a type the source never wrote.
    fn discarded(&mut self, statement: &Statement, value: &Type) {
        if let StatementKind::Expr(left) = &statement.kind {
            self.discards.push((value.clone(), left.span));
        }
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
            StatementKind::Discard(expr) => {
                self.expr(expr)?;
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
            self.adds_to(assigned, target.span);
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
        self.block(&walked.body, Gives::Nothing)?;
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
            ExprKind::Unary { operator, operand } => self.unary(*operator, operand, expr.span),
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
            ExprKind::List(elements) => self.written_list(elements),
            ExprKind::Record { base, fields } => self.record(base, fields, expr.span),
            ExprKind::If(chain) => self.if_expr(chain),
            ExprKind::Match(matched) => self.match_expr(matched),
        }
    }

    /// A written list, which is a `List` of the one type every element it writes shares.
    ///
    /// Each element is met against one variable, so the first settles what the list holds and
    /// every one after it is held to that. A list of nothing leaves the variable free, and
    /// whatever the list itself is unified with is what settles it.
    fn written_list(&mut self, elements: &[Expr]) -> Result<Type, TypeError> {
        let item = self.table.fresh();
        for element in elements {
            let found = self.expr(element)?;
            self.expect(&item, &found, element.span)?;
        }
        Ok(Type::list(item))
    }

    fn call(&mut self, callee: &Expr, arguments: &Arguments, at: Span) -> Result<Type, TypeError> {
        let signature = self.expr(callee)?;
        let passed = arguments.values();
        let mut given = Vec::new();
        for argument in &passed {
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
        for ((wanted, found), argument) in parameters.iter().zip(&given).zip(&passed) {
            self.expect(wanted, found, argument.span)?;
        }
        self.named_as_declared(callee, arguments, at)?;
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

    /// `?` hands the case with nothing to go on back, and leaves what the other case carries.
    ///
    /// `docs/design.md` section 5 states the rule: each kind lands in a function that gives back
    /// the same kind, so nothing is converted. A `?` that neither the function's result nor what
    /// it is written on has settled yet waits for the body to say, as a field lookup does.
    fn propagated(&mut self, inner: &Expr, at: Span) -> Result<Type, TypeError> {
        let found = self.expr(inner)?;
        let waiting = Propagation {
            found,
            held: self.table.fresh(),
            inner: inner.span,
            at,
        };
        let held = waiting.held.clone();
        match self.propagates(&waiting.found) {
            Some(kind) => self.propagate(waiting, kind)?,
            None => self.propagations.push(waiting),
        }
        Ok(held)
    }

    /// Which of the two kinds this `?` propagates, which is the one its function gives back.
    ///
    /// What the `?` is written on answers where the function has not, which is how a body with
    /// no signature above it still divides through `?`.
    fn propagates(&self, found: &Type) -> Option<Propagated> {
        propagated(&self.table.solved(&self.result))
            .or_else(|| propagated(&self.table.solved(found)))
    }

    fn if_expr(&mut self, chain: &IfExpr) -> Result<Type, TypeError> {
        let mut blocks = Vec::new();
        for branch in &chain.branches {
            self.condition(&branch.condition)?;
            blocks.push((
                self.block(&branch.block, Gives::ItsValue)?,
                branch.block.span,
            ));
        }
        let result = match &chain.otherwise {
            Some(block) => self.block(block, Gives::ItsValue)?,
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
    ///
    /// A name written over a constrained type parameter asks its trait of whatever this use
    /// settles that parameter on, and a trait's method asks its own trait the same way.
    fn value(&mut self, name: &Name) -> Type {
        let key = self.key_of(name);
        let scheme = self.scheme(&key);
        let (found, asked) = scheme.at_one_use(&mut self.table);
        let how = match self.environment.of_a_trait(&key) {
            Some(_) => Asked::Method,
            None => Asked::Constraint,
        };
        for required in asked {
            self.requirements.push(Requirement {
                required,
                written: name.span,
                how,
            });
        }
        found
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
        let required = shell.required().to_vec();
        self.environment
            .bind(key, Scheme::over(quantified, body).requiring(required));
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
        for operated in &self.operated {
            self.table.unsettled(&operated.at, &mut held);
        }
        for requirement in &self.requirements {
            self.table.unsettled(&requirement.required.at, &mut held);
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
pub(crate) struct Binary<'a> {
    pub(crate) operator: BinaryOperator,
    pub(crate) left: &'a Expr,
    pub(crate) right: &'a Expr,
    /// The whole expression, which is what a refused operator points the reader at.
    pub(crate) at: Span,
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

/// One `?`, and what it was written on, waiting on the kind its function gives back.
pub(crate) struct Propagation {
    /// The type of what the `?` is written on, which is one of the two kinds.
    pub(crate) found: Type,
    /// The type the `?` leaves behind, which is what that kind carries.
    pub(crate) held: Type,
    /// Where what the `?` is written on is written, which a mismatch there points at.
    pub(crate) inner: Span,
    /// Where the `?` is written, which a function of the wrong kind is reported at.
    pub(crate) at: Span,
}

/// One trait a use must answer for, at the type that use asked it of.
pub(crate) struct Requirement {
    pub(crate) required: Required,
    /// Where the use is written, which is what an unanswered trait is reported against.
    pub(crate) written: Span,
    pub(crate) how: Asked,
}

/// How a trait came to be asked for, which is what an unanswered one is worded by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Asked {
    /// A call of one of the trait's methods, which reaches the instance for that type.
    Method,
    /// A use of something whose type parameter the declaration constrained by the trait.
    Constraint,
    /// An operator, which is the method of the trait `docs/specs/operators.md` says it is.
    Operator(&'static str),
}

/// Which case a `?` hands back, neither of which ever becomes the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Propagated {
    /// An `Option`, whose `None` says there is nothing and has nothing more to say.
    Absence,
    /// A `Result`, whose `Err` says what the caller could not have worked out.
    Failure,
}

/// Which kind `settled` is, when it is one of the two a `?` propagates.
pub(crate) fn propagated(settled: &Type) -> Option<Propagated> {
    let Type::Named { name, .. } = settled else {
        return None;
    };
    match name.as_str() {
        OPTION => Some(Propagated::Absence),
        RESULT => Some(Propagated::Failure),
        _ => None,
    }
}

/// What becomes of a block's value, which decides whether its last statement is discarded.
///
/// A function body gives its value to the result the function declares, and an `if` or a `match`
/// gives its value to the expression it is written in. A `for` body gives its value to nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Gives {
    ItsValue,
    Nothing,
}
