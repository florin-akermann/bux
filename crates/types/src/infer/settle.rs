//! What a function settles once its body has been walked.
//!
//! A field waits on the type it is reached through, and an operator waits on the type of what it
//! is written over. Each of them is answered when the function they are written in has been
//! inferred, so the report lands where the source wrote it.

use std::mem;

use lumen_ast::Name;

use crate::environment::{self, Key};
use crate::error::{TypeError, TypeErrorKind};
use crate::infer::{Asked, Inference, Propagated, Propagation, Requirement, labelled};
use crate::scheme::Required;
use crate::supplied;
use crate::surface::{GenericUse, Offered};
use crate::types::Type;

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

    /// The operators of one function, each asking its trait of the type its operands settled on.
    ///
    /// Every operator is a trait method, which `docs/specs/operators.md` states, so an operator
    /// asks its trait exactly as a call of one of that trait's methods does. They settle after
    /// the declared result has had its say, because the result is often the only thing that says
    /// a `+` joins two `String`s. An operator still untyped by then is over `Int`s, which is the
    /// one default the language keeps, and the default is taken before the trait is asked.
    pub(crate) fn settle_operators(&mut self) -> Result<(), TypeError> {
        for operated in mem::take(&mut self.operated) {
            if matches!(self.table.shallow(&operated.at), Type::Var(_)) {
                self.expect(&Type::int(), &operated.at, operated.written)?;
            }
            self.requirements.push(Requirement {
                required: Required {
                    trait_name: operated.of.to_owned(),
                    at: operated.at,
                },
                written: operated.written,
                how: Asked::Operator(operated.written_as),
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
            if let Some(kind) = left_behind_by(&requirement.how, of, &at) {
                return Err(TypeError::at(requirement.written, kind));
            }
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
    /// They settle after the operators, so a discarded `a == b` is named as the `Bool` it is
    /// rather than as a type nothing had settled yet. A statement inference never settled takes
    /// `()`, as an unsettled operator takes `Int`, so `todo("not yet")` stands as a statement.
    pub(crate) fn settle_discards(&mut self) -> Result<(), TypeError> {
        for (found, at) in mem::take(&mut self.discards) {
            let settled = self.table.shallow(&found);
            if matches!(settled, Type::Var(_)) {
                self.expect(&Type::Unit, &found, at)?;
            } else if let Some(kind) = not_discardable(&self.table.solved(&settled)) {
                return Err(TypeError::at(at, kind));
            }
        }
        Ok(())
    }

    pub(crate) fn look_up(&mut self, lookup: Lookup) -> Result<(), TypeError> {
        let through = self.table.solved(&lookup.through);
        if let Type::Module(module) = &through {
            return self.inside(&module.clone(), &lookup);
        }
        let field = lookup.field;
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
    ///
    /// Version 0.1 has no function value, so a function of a module is written where a call
    /// writes it and nowhere else; a variant it declares that carries nothing is a value and is
    /// written as the name alone, which `docs/specs/modules.md` states.
    fn inside(&mut self, module: &str, lookup: &Lookup) -> Result<(), TypeError> {
        let field = &lookup.field;
        let declared = if supplied::supplies(module) {
            supplied::declared(module, &field.text)
        } else {
            self.reached(module, field)
        };
        let Some(declared) = declared else {
            let kind = TypeErrorKind::NotInModule {
                module: module.to_owned(),
                name: field.text.clone(),
            };
            return Err(TypeError::at(field.span, kind));
        };
        if lookup.how == Reached::AsAValue && matches!(declared, Type::Function { .. }) {
            let kind = TypeErrorKind::NotCalled {
                module: module.to_owned(),
                name: field.text.clone(),
            };
            return Err(TypeError::at(field.span, kind));
        }
        self.expect(&lookup.found, &declared, field.span)
    }

    /// The type a loaded module gives `field`, freshly at this use, when it offers one at all.
    ///
    /// A signature naming a type that module declares names it as this module writes it, which
    /// `docs/specs/modules.md` states: the surface is offered under the name it is imported by.
    ///
    /// A generic is offered like any other, and what this use settled its written type parameters
    /// on is noted: `docs/specs/codegen.md` writes one method per set, and the module declaring it
    /// writes the set only once something says which one.
    fn reached(&mut self, module: &str, field: &Name) -> Option<Type> {
        let imported = self.imported;
        let surface = imported.surface(module)?;
        let Some(offered) = surface.function(&field.text) else {
            return self.built_inside(module, field);
        };
        let (Offered::Plain(scheme) | Offered::Generic(scheme)) = offered;
        let use_of_it = scheme.clone().at_one_use(&mut self.table);
        let how = how_it_is_asked(offered, module, field);
        for required in use_of_it.required {
            self.requirements.push(Requirement {
                required,
                written: field.span,
                how: how.clone(),
            });
        }
        if let Offered::Generic(_) = offered {
            let reaching = GenericUse::reaching(use_of_it.settling, use_of_it.written_as);
            self.generics_reached.insert(field.span, reaching);
        }
        Some(use_of_it.found)
    }

    /// The constructor `module` offers as `field`, which builds a value of a type it declares.
    ///
    /// A variant carrying nothing is written as the name alone and one carrying something is
    /// called, so both are reached here: a call reads the type of its callee off this too.
    fn built_inside(&mut self, module: &str, field: &Name) -> Option<Type> {
        let key = Key::reached(&format!("{module}.{}", field.text));
        let scheme = self.environment.scheme(&key).cloned()?;
        Some(scheme.instantiate(&mut self.table))
    }
}

/// How the traits a use of `offered` must answer for came to be asked.
///
/// A generic is written by the module that declares it, which `docs/specs/codegen.md` states, so
/// the body a constraint of one is asked on behalf of is that module's rather than this one's.
fn how_it_is_asked(offered: &Offered, module: &str, field: &Name) -> Asked {
    match offered {
        Offered::Plain(_) => Asked::Constraint,
        Offered::Generic(_) => Asked::OfAnotherModule {
            module: module.to_owned(),
            name: field.text.clone(),
        },
    }
}

/// What is wrong with asking another module's generic for an instance that does not reach it.
///
/// A trait and its instances stay where they are declared, which `docs/specs/modules.md` states,
/// and a module offers neither, so the prelude's are the only instances this module can know
/// another one has. A constraint on a generic that module writes is answered in its body, so a
/// use settling that type parameter anywhere else asks for a body nothing could write.
///
/// It is refused before the instance is looked for at all: an instance this module reaches is no
/// answer either, and saying to write one would be saying to write what would then be refused.
fn left_behind_by(asked: &Asked, of: &str, at: &Type) -> Option<TypeErrorKind> {
    let Asked::OfAnotherModule { module, name } = asked else {
        return None;
    };
    let reached = match at {
        Type::Named { name, .. } => environment::of_the_prelude(of, name),
        _ => false,
    };
    (!reached).then(|| TypeErrorKind::InstanceStaysInItsModule {
        module: module.clone(),
        name: name.clone(),
        of: of.to_owned(),
        at: at.clone(),
    })
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
    pub(crate) how: Reached,
}

/// Where a name reached through a `.` is written, which says what may be found there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reached {
    /// The name of a call, which is the one place a function of a module is written.
    AsACall,
    /// Anywhere else, where a function is nothing this version can hold.
    AsAValue,
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
    let of = requirement.required.trait_name.clone();
    let kind = match requirement.how {
        Asked::Operator(written_as) => TypeErrorKind::NoOperator { written_as, of, at },
        Asked::Method | Asked::Constraint | Asked::OfAnotherModule { .. } => {
            TypeErrorKind::NoInstance { of, at }
        }
    };
    TypeError::at(requirement.written, kind)
}
