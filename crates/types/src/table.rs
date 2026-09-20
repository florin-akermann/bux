//! What each variable has been settled to, and where a new one comes from.

use std::collections::{HashMap, HashSet};

use crate::types::{Type, TypeVar};

/// What a whole number nothing has settled stands for, where one is followed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Standing {
    /// The variable it is, which the code around it may still settle on a type.
    AsItIs,
    /// The `Int` it defaults to, which `docs/specs/literals.md` keeps as the one default.
    AsTheIntItDefaultsTo,
}

/// The variables inference has made, and what each one has been settled to.
#[derive(Debug, Default)]
pub(crate) struct Table {
    settled: HashMap<TypeVar, Type>,
    /// The variables standing for a whole number written in the source, which settle on less
    /// than an ordinary variable does and read as `Int` while nothing has settled them.
    whole_numbers: HashSet<TypeVar>,
    /// The types a whole number is written at, which `docs/specs/literals.md` says is every type
    /// with an instance of `IntegerLiteral`.
    written_at: HashSet<String>,
    made: u32,
}

impl Table {
    /// A variable no other place in the program has.
    pub(crate) fn fresh(&mut self) -> Type {
        Type::Var(self.made_up())
    }

    /// A variable standing for a whole number, which settles only where one may be written.
    pub(crate) fn fresh_whole_number(&mut self) -> Type {
        let var = self.made_up();
        self.whole_numbers.insert(var);
        Type::Var(var)
    }

    /// Says which types a whole number may be written at, once every instance has been read.
    pub(crate) fn whole_numbers_are_written_at(&mut self, types: impl Iterator<Item = String>) {
        self.written_at.extend(types);
    }

    /// Makes `var` stand for a whole number, which is what meeting one does to it.
    pub(crate) fn note_a_whole_number(&mut self, var: TypeVar) {
        self.whole_numbers.insert(var);
    }

    /// Whether a whole number is written at `to`, which is whether `to` has the instance.
    pub(crate) fn takes_a_whole_number(&self, to: &Type) -> bool {
        match to {
            Type::Named { name, .. } => self.written_at.contains(name),
            _ => false,
        }
    }

    /// Settles `var` on `to`, which the occurs check has already cleared.
    pub(crate) fn settle(&mut self, var: TypeVar, to: Type) {
        self.settled.insert(var, to);
    }

    /// `of` with every variable anywhere in it followed to whatever it stands for.
    pub(crate) fn solved(&self, of: &Type) -> Type {
        self.followed(of, Standing::AsItIs)
    }

    /// The same, with a whole number nothing has settled read as the `Int` it defaults to.
    ///
    /// That is what a refusal owes its reader: a whole number met a type that takes none, and the
    /// reader wrote a number rather than the variable inference gave it. Inference itself follows
    /// the variable instead, because the code around it may still settle it on something else.
    pub(crate) fn read(&self, of: &Type) -> Type {
        self.followed(of, Standing::AsTheIntItDefaultsTo)
    }

    /// `of` followed to whatever it stands for, with `standing` saying what an unsettled whole
    /// number stands for.
    fn followed(&self, of: &Type, standing: Standing) -> Type {
        match self.shallow(of) {
            Type::Var(var)
                if standing == Standing::AsTheIntItDefaultsTo && self.is_a_whole_number(var) =>
            {
                Type::int()
            }
            Type::Named { name, arguments } => Type::Named {
                name,
                arguments: self.each(&arguments, standing),
            },
            Type::Function { parameters, result } => Type::function(
                self.each(&parameters, standing),
                self.followed(&result, standing),
            ),
            settled => settled,
        }
    }

    fn each(&self, types: &[Type], standing: Standing) -> Vec<Type> {
        types
            .iter()
            .map(|one| self.followed(one, standing))
            .collect()
    }

    /// Every variable `of` still holds once it is solved, each named once.
    pub(crate) fn unsettled(&self, of: &Type, into: &mut Vec<TypeVar>) {
        match self.shallow(of) {
            Type::Var(var) => {
                if !into.contains(&var) {
                    into.push(var);
                }
            }
            Type::Named { arguments, .. } => self.each_unsettled(&arguments, into),
            Type::Function { parameters, result } => {
                self.each_unsettled(&parameters, into);
                self.unsettled(&result, into);
            }
            Type::Parameter(_) | Type::Module(_) | Type::Unit => {}
        }
    }

    /// `of` with its outermost variable followed to whatever it stands for.
    pub(crate) fn shallow(&self, of: &Type) -> Type {
        let Type::Var(var) = of else {
            return of.clone();
        };
        match self.settled.get(var) {
            Some(settled) => self.shallow(settled),
            None => of.clone(),
        }
    }

    /// Whether `var` stands for a whole number rather than for anything at all.
    pub(crate) fn is_a_whole_number(&self, var: TypeVar) -> bool {
        self.whole_numbers.contains(&var)
    }

    /// The next variable there is, which no other place in the program has.
    fn made_up(&mut self) -> TypeVar {
        let var = TypeVar(self.made);
        self.made += 1;
        var
    }

    fn each_unsettled(&self, types: &[Type], into: &mut Vec<TypeVar>) {
        for one in types {
            self.unsettled(one, into);
        }
    }
}
