//! What a function settles once its body has been walked.
//!
//! A field waits on the type it is reached through, and an addition and a comparison wait on
//! the type of what they are given. Each of them is answered when the function they are
//! written in has been inferred, so the report lands where the source wrote it.

use std::mem;

use lumen_ast::{Name, Span};

use crate::error::{TypeError, TypeErrorKind};
use crate::infer::{Asked, Inference, Propagated, Propagation, Requirement, labelled};
use crate::scheme::Required;
use crate::supplied;
use crate::surface::Offered;
use crate::types::{EQ, Type};

impl Inference<'_> {
    /// The fields of one function, now that its body has said what they are reached through.
    ///
    /// They settle before the body meets the declared result, so a field that disagrees with what
    /// its record declares is reported where it is written rather than as a body of the wrong type.
    pub(crate) fn look_up_fields(&mut self) -> Result<(), TypeError> {
        for lookup in mem::take(&mut self.lookups) {
            self.look_up(lookup)?;
        }
        Ok(())
    }

    /// The `?`s of one function that nothing had settled when they were walked past.
    ///
    /// They settle after the body has met the declared result, which is the last thing that can
    /// say what a function with no signature gives back. A function that recurses through its
    /// own `?` is the reason: nothing but its body ever says which kind it propagates. One that
    /// still says nothing propagates a `Result`, which is the kind a `?` with no answer is
    /// reported against.
    pub(crate) fn settle_propagations(&mut self) -> Result<(), TypeError> {
        for waiting in mem::take(&mut self.propagations) {
            let kind = self
                .propagates(&waiting.found)
                .unwrap_or(Propagated::Failure);
            self.propagate(waiting, kind)?;
        }
        Ok(())
    }

    /// One `?`, held to the kind it propagates on both sides: what it is given, and what it
    /// leaves the function.
    ///
    /// The two sides of a `Result` share an error type, so a `?` never propagates into a
    /// function that names another one. An `Option` carries nothing to share, so a `None` goes
    /// into any function that gives one back.
    pub(crate) fn propagate(
        &mut self,
        waiting: Propagation,
        kind: Propagated,
    ) -> Result<(), TypeError> {
        let (written_on, given_back) = match kind {
            Propagated::Absence => (Type::option(waiting.held), Type::option(self.table.fresh())),
            Propagated::Failure => {
                let error = self.table.fresh();
                (
                    Type::result(waiting.held, error.clone()),
                    Type::result(self.table.fresh(), error),
                )
            }
        };
        self.expect(&written_on, &waiting.found, waiting.inner)?;
        self.expect(&self.result.clone(), &given_back, waiting.at)
    }

    /// The additions of one function, each an addition of `Int`s unless something said otherwise.
    ///
    /// They settle last, after the declared result has had its say, because the result is often
    /// the only thing that says an addition joins two `String`s.
    pub(crate) fn settle_additions(&mut self) -> Result<(), TypeError> {
        let waiting = mem::take(&mut self.additions);
        self.settled(waiting, &Type::int(), not_addable)
    }

    /// The comparisons of one function, each between two values of a type that has `Eq`.
    ///
    /// `==` is `Eq`, which `docs/design.md` section 8 states, so a comparison asks the trait of
    /// the type it compares exactly as a call of `equals` does. A comparison still untyped once
    /// its function is inferred is a comparison of `Int`s, as an addition still untyped is an
    /// addition of them, so the default is settled before the trait is asked.
    pub(crate) fn settle_equalities(&mut self) -> Result<(), TypeError> {
        for (found, at) in mem::take(&mut self.equalities) {
            if matches!(self.table.shallow(&found), Type::Var(_)) {
                self.expect(&Type::int(), &found, at)?;
            }
            self.requirements.push(Requirement {
                required: Required {
                    trait_name: EQ.to_owned(),
                    at: found,
                },
                written: at,
                how: Asked::Comparison,
            });
        }
        Ok(())
    }

    /// The traits the current function asked for, each answered by an instance or refused.
    ///
    /// They settle after every default has been taken, so a comparison of two `1`s asks `Eq` of
    /// `Int` rather than of a type nothing had settled. `docs/specs/traits.md` states what each
    /// of the three kinds of type answers with.
    pub(crate) fn settle_requirements(&mut self) -> Result<(), TypeError> {
        for requirement in mem::take(&mut self.requirements) {
            let at = self.table.solved(&requirement.required.at);
            let of = &requirement.required.trait_name;
            if !self.answers(of, &at) {
                return Err(unanswered(&requirement, at));
            }
            if requirement.how == Asked::Method {
                self.methods_at.insert(requirement.written, at);
            }
        }
        Ok(())
    }

    /// Whether the trait `of` is answered at `at`, which is where a constraint is resolved.
    ///
    /// A named type is answered by the one instance there is for it; a type parameter of the
    /// function being inferred is answered by the constraint that parameter declares; nothing
    /// else names an instance at all.
    fn answers(&self, of: &str, at: &Type) -> bool {
        match at {
            Type::Named { name, .. } => self.environment.has_instance(of, name),
            Type::Parameter(_) => self
                .promised
                .iter()
                .any(|promise| promise.trait_name == *of && promise.at == *at),
            _ => false,
        }
    }

    /// The statements of one function that nothing takes the value of.
    ///
    /// They settle after the comparisons, so a discarded `a == b` is named as the `Bool` it is
    /// rather than as a type nothing had settled yet. A statement inference never settled takes
    /// `()`, as an unsettled addition takes `Int`, so `todo("not yet")` stands as a statement.
    pub(crate) fn settle_discards(&mut self) -> Result<(), TypeError> {
        let waiting = mem::take(&mut self.discards);
        self.settled(waiting, &Type::Unit, not_discardable)
    }

    /// Each type that was waiting on the function it is written in, now that the function is done.
    ///
    /// A type nothing settled takes `default`, which is the one default the language has and the
    /// reason `1 + 1` is an addition of `Int`s. A type that did settle is put to `refused`, which
    /// says what is wrong with it or that nothing is.
    fn settled(
        &mut self,
        waiting: Vec<(Type, Span)>,
        default: &Type,
        refused: fn(&Type) -> Option<TypeErrorKind>,
    ) -> Result<(), TypeError> {
        for (found, at) in waiting {
            let settled = self.table.shallow(&found);
            if matches!(settled, Type::Var(_)) {
                self.expect(default, &found, at)?;
            } else if let Some(kind) = refused(&self.table.solved(&settled)) {
                return Err(TypeError::at(at, kind));
            }
        }
        Ok(())
    }

    pub(crate) fn look_up(&mut self, lookup: Lookup) -> Result<(), TypeError> {
        let through = self.table.solved(&lookup.through);
        let field = lookup.field;
        if let Type::Module(module) = &through {
            return self.inside(module, &field, &lookup.found);
        }
        let Type::Named { name, .. } = &through else {
            return Err(unreachable_field(&through, &field));
        };
        let Some(key) = self.environment.record(name).cloned() else {
            return Err(unreachable_field(&through, &field));
        };
        let labels = self.environment.labels(&key).to_vec();
        let scheme = self.scheme(&key);
        let Type::Function { parameters, result } = scheme.instantiate(&mut self.table) else {
            return Err(unreachable_field(&through, &field));
        };
        self.expect(&through, &result, field.span)?;
        let index = labelled(&labels, &field, &self.table.solved(&result))?;
        self.expect(&lookup.found, &parameters[index].clone(), field.span)
    }

    /// A name reached inside a module, which the compiler supplies or loading read from a file.
    fn inside(&mut self, module: &str, field: &Name, found: &Type) -> Result<(), TypeError> {
        let declared = if supplied::supplies(module) {
            supplied::declared(module, &field.text)
        } else {
            self.reached(module, field)?
        };
        let Some(declared) = declared else {
            let kind = TypeErrorKind::NotInModule {
                module: module.to_owned(),
                name: field.text.clone(),
            };
            return Err(TypeError::at(field.span, kind));
        };
        self.expect(found, &declared, field.span)
    }

    /// The type a loaded module gives `field`, freshly at this use, when it offers one at all.
    ///
    /// A signature naming a type that module declares offers nothing this one can write, which
    /// `docs/specs/modules.md` states and refuses here rather than where it is declared.
    fn reached(&mut self, module: &str, field: &Name) -> Result<Option<Type>, TypeError> {
        let imported = self.imported;
        let Some(surface) = imported.surface(module) else {
            return Ok(None);
        };
        let scheme = match surface.function(&field.text) {
            None => return Ok(None),
            Some(Offered::Generic) => {
                let kind = TypeErrorKind::GenericThroughModule {
                    module: module.to_owned(),
                    name: field.text.clone(),
                };
                return Err(TypeError::at(field.span, kind));
            }
            Some(Offered::Plain(scheme)) => scheme,
        };
        if let Some(kept) = surface.kept_to_itself(scheme) {
            let kind = TypeErrorKind::TypeOfAnotherModule {
                module: module.to_owned(),
                name: field.text.clone(),
                declared: kept.to_owned(),
            };
            return Err(TypeError::at(field.span, kind));
        }
        Ok(Some(scheme.clone().instantiate(&mut self.table)))
    }
}

/// What is wrong with adding two of `found`, which is that `+` joins `Int`s or `String`s.
fn not_addable(found: &Type) -> Option<TypeErrorKind> {
    let addable = *found == Type::int() || *found == Type::string();
    (!addable).then(|| TypeErrorKind::NotAddable(found.clone()))
}

/// What is wrong with leaving a `found` behind, which is that nothing is there to take it.
fn not_discardable(found: &Type) -> Option<TypeErrorKind> {
    (*found != Type::Unit).then(|| TypeErrorKind::Discarded(found.clone()))
}

/// A field waiting on the type it is reached through.
pub(crate) struct Lookup {
    pub(crate) through: Type,
    pub(crate) field: Name,
    pub(crate) found: Type,
}

/// A field reached through something that has no fields, or through a type nothing settled.
///
/// A module is never one of them: every module in scope is supplied or loaded, so a name
/// reached inside one is answered by what it declares.
fn unreachable_field(through: &Type, field: &Name) -> TypeError {
    let kind = if matches!(through, Type::Var(_)) {
        TypeErrorKind::UnknownReceiver(field.text.clone())
    } else {
        TypeErrorKind::UnknownField {
            of: through.clone(),
            field: field.text.clone(),
        }
    };
    TypeError::at(field.span, kind)
}

/// The refusal a trait nothing answers amounts to, worded by how it came to be asked.
fn unanswered(requirement: &Requirement, at: Type) -> TypeError {
    let kind = match requirement.how {
        Asked::Comparison => TypeErrorKind::NotEquatable(at),
        Asked::Method | Asked::Constraint => TypeErrorKind::NoInstance {
            of: requirement.required.trait_name.clone(),
            at,
        },
    };
    TypeError::at(requirement.written, kind)
}
