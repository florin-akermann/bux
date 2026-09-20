//! What a trait declares, and what the instance of a type owes it.
//!
//! `docs/specs/traits.md` states the three shapes this resolves: a trait's signatures, read
//! against the one type parameter it is written over; an instance, read against the trait it
//! names; and a constraint, which names a trait where a type parameter is introduced.

use std::collections::HashSet;

use lumen_ast::{InstanceDeclaration, Name, Signature, TraitDeclaration, TypeParameter};

use crate::definition::{DefinitionKind, Origin};
use crate::error::{ResolveError, ResolveErrorKind};
use crate::prelude;
use crate::resolve::{Resolved, Resolver};

impl Resolver {
    /// A trait names itself among the types, and names each of its methods among the values.
    ///
    /// An instance declares nothing: two instances each write `equals`, and the name both of
    /// them answer for is the trait's, which `docs/specs/traits.md` states.
    pub(super) fn declare_trait(&mut self, declaration: &TraitDeclaration) -> Resolved {
        self.introduce_type(&declaration.name, DefinitionKind::Trait)?;
        for method in &declaration.methods {
            self.introduce_value(&method.name, DefinitionKind::TraitMethod)?;
        }
        let methods = declaration
            .methods
            .iter()
            .map(|method| method.name.text.clone())
            .collect();
        self.declared_traits
            .insert(declaration.name.text.clone(), methods);
        Ok(())
    }

    /// A trait's signatures, each read against the one type parameter the trait is written over.
    pub(super) fn trait_declaration(&mut self, declaration: &TraitDeclaration) -> Resolved {
        self.types.enter();
        self.introduce_type(&declaration.parameter, DefinitionKind::TypeParameter)?;
        for method in &declaration.methods {
            self.signature(method)?;
        }
        self.types.leave();
        Ok(())
    }

    /// One signature: the types it names, and the parameter names it takes while it names them.
    fn signature(&mut self, method: &Signature) -> Resolved {
        self.values.enter();
        for parameter in &method.parameters {
            if let Some(type_ref) = &parameter.type_ref {
                self.type_ref(type_ref)?;
            }
            self.introduce_value(&parameter.name, DefinitionKind::Parameter)?;
        }
        if let Some(result) = &method.result {
            self.type_ref(result)?;
        }
        self.values.leave();
        Ok(())
    }

    /// An instance: the trait it is for, the type it is for, and a body per declared method.
    pub(super) fn instance(&mut self, declaration: &InstanceDeclaration) -> Resolved {
        let declared = self.trait_named(&declaration.trait_name)?;
        self.named_type(&declaration.for_type)?;
        self.claimed(declaration)?;
        writes_declared_names_once(declaration, &declared)?;
        writes_every_one(declaration, &declared)?;
        for method in &declaration.methods {
            self.function(method)?;
        }
        Ok(())
    }

    /// Records that this trait now has an instance for this type, refusing a second one.
    fn claimed(&mut self, declaration: &InstanceDeclaration) -> Resolved {
        let of = declaration.trait_name.text.clone();
        let for_type = declaration.for_type.text.clone();
        if self.instances.insert((of.clone(), for_type.clone())) {
            return Ok(());
        }
        let kind = ResolveErrorKind::InstanceTwice { of, for_type };
        Err(ResolveError::at(&declaration.for_type, kind))
    }

    /// The trait a type parameter is constrained by, when the author wrote one.
    ///
    /// The constraint is read after every type parameter is in scope, so `<T: Eq<T>>` names `T`
    /// where the declaration has just introduced it.
    pub(super) fn constrained(&mut self, parameter: &TypeParameter) -> Resolved {
        let Some(constraint) = &parameter.constraint else {
            return Ok(());
        };
        self.trait_named(&constraint.name)?;
        self.type_ref(&constraint.argument)
    }

    /// The names of the methods the trait `name` declares, refusing anything that is no trait.
    fn trait_named(&mut self, name: &Name) -> Result<Vec<String>, ResolveError> {
        self.use_type(name)?;
        let found = self.types.look_up(&name.text);
        if found.map(|one| one.kind) != Some(DefinitionKind::Trait) {
            let kind = ResolveErrorKind::NotATrait(name.text.clone());
            return Err(ResolveError::at(name, kind));
        }
        match found.map(|one| one.origin) {
            Some(Origin::Prelude) => Ok(named(prelude::methods_of(&name.text).unwrap_or(&[]))),
            _ => Ok(self.methods_declared_at(name)),
        }
    }

    /// The methods of the trait this module declares at `name`, read off the declaration.
    fn methods_declared_at(&self, name: &Name) -> Vec<String> {
        self.declared_traits
            .get(&name.text)
            .cloned()
            .unwrap_or_default()
    }
}

/// The instances the compiler supplies, which a module writing one of them again is refused by.
pub(super) fn supplied_instances() -> HashSet<(String, String)> {
    prelude::EQUATABLE
        .iter()
        .map(|for_type| (prelude::EQ.to_owned(), (*for_type).to_owned()))
        .collect()
}

/// The names `written` holds, as the owned strings a refusal is worded with.
fn named(written: &[&str]) -> Vec<String> {
    written.iter().map(|one| (*one).to_owned()).collect()
}

/// Refuses the first method an instance writes that its trait never declared, or writes twice.
///
/// The two are one walk over the names an instance writes, because an instance's method names
/// are its trait's: a name here is one the trait declares and has yet to be written, or it is a
/// refusal. A name written twice would put no second name in scope, so one of the two bodies
/// would be reached and the other lost.
fn writes_declared_names_once(declaration: &InstanceDeclaration, declared: &[String]) -> Resolved {
    let mut written = HashSet::new();
    for method in &declaration.methods {
        let name = method.name.text.clone();
        let of = declaration.trait_name.text.clone();
        let refusal = if !declared.contains(&name) {
            ResolveErrorKind::MethodUndeclared { of, method: name }
        } else if written.insert(name.clone()) {
            continue;
        } else {
            ResolveErrorKind::MethodTwice { of, method: name }
        };
        return Err(ResolveError::at(&method.name, refusal));
    }
    Ok(())
}

/// Refuses the first method an instance's trait declares that the instance does not write.
fn writes_every_one(declaration: &InstanceDeclaration, declared: &[String]) -> Resolved {
    let written = |wanted: &&String| {
        declaration
            .methods
            .iter()
            .any(|one| one.name.text == **wanted)
    };
    let Some(missing) = declared.iter().find(|wanted| !written(wanted)) else {
        return Ok(());
    };
    let kind = ResolveErrorKind::MethodMissing {
        of: declaration.trait_name.text.clone(),
        method: missing.clone(),
    };
    Err(ResolveError::at(&declaration.trait_name, kind))
}
