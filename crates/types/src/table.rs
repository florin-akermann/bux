//! What each variable has been settled to, and where a new one comes from.

use std::collections::HashMap;

use crate::types::{Type, TypeVar};

/// The variables inference has made, and what each one has been settled to.
#[derive(Debug, Default)]
pub(crate) struct Table {
    settled: HashMap<TypeVar, Type>,
    made: u32,
}

impl Table {
    /// A variable no other place in the program has.
    pub(crate) fn fresh(&mut self) -> Type {
        let var = TypeVar(self.made);
        self.made += 1;
        Type::Var(var)
    }

    /// Settles `var` on `to`, which the occurs check has already cleared.
    pub(crate) fn settle(&mut self, var: TypeVar, to: Type) {
        self.settled.insert(var, to);
    }

    /// `of` with every variable anywhere in it followed to whatever it stands for.
    pub(crate) fn solved(&self, of: &Type) -> Type {
        match self.shallow(of) {
            Type::Named { name, arguments } => Type::Named {
                name,
                arguments: self.each(&arguments),
            },
            Type::Function { parameters, result } => {
                Type::function(self.each(&parameters), self.solved(&result))
            }
            settled => settled,
        }
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

    fn each(&self, types: &[Type]) -> Vec<Type> {
        types.iter().map(|one| self.solved(one)).collect()
    }

    fn each_unsettled(&self, types: &[Type], into: &mut Vec<TypeVar>) {
        for one in types {
            self.unsettled(one, into);
        }
    }
}
