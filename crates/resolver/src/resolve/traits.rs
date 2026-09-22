//! What a trait declares, and what the instance of a type owes it.
//!
//! `docs/specs/traits.md` states the three shapes this resolves: a trait's signatures, read
//! against the one type parameter it is written over; an instance, read against the trait it
//! names; and a constraint, which names a trait where a type parameter is introduced.

use std::collections::HashSet;

use lumen_ast::{DeriveDeclaration, InstanceDeclaration, Name, Signature};
use lumen_ast::{TraitDeclaration, TypeParameter};

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
    ///
    /// The type it is for may be written with arguments, and each of them names a type parameter
    /// the instance declares, which `docs/specs/traits.md` states. Each method is written over
    /// those parameters, so resolving a method introduces them exactly as a generic function does.
    pub(super) fn instance(&mut self, declaration: &InstanceDeclaration) -> Resolved {
        let declared = self.trait_named(&declaration.trait_name)?;
        self.named_type(&declaration.for_type)?;
        self.claimed(&declaration.trait_name, &declaration.for_type)?;
        writes_declared_names_once(declaration, &declared)?;
        writes_every_one(declaration, &declared)?;
        written_over_its_parameters(declaration)?;
        for method in &declaration.methods {
            self.function(method)?;
        }
        Ok(())
    }

    /// A derive: each trait it names, and the type the compiler writes the instances for.
    ///
    /// A derive writes an instance, so it claims what an instance claims and is refused where an
    /// instance would be. `docs/specs/derive.md` states which traits a type derives.
    pub(super) fn derive(&mut self, declaration: &DeriveDeclaration) -> Resolved {
        self.named_type(&declaration.for_type)?;
        self.declared_here(&declaration.for_type)?;
        for named in &declaration.traits {
            self.trait_named(named)?;
            derivable(named)?;
            self.claimed(named, &declaration.for_type)?;
        }
        Ok(())
    }

    /// Refuses a derive for a type this module does not declare, which it writes nothing for.
    ///
    /// What a derive writes follows from the declaration it names, so the declaration has to be
    /// here to be read. A prelude type has the instances the prelude ships and no others.
    fn declared_here(&mut self, for_type: &Name) -> Resolved {
        let origin = self.types.look_up(&for_type.text).map(|one| one.origin);
        if matches!(origin, Some(Origin::Declared(_))) {
            return Ok(());
        }
        let kind = ResolveErrorKind::NotDeclaredHere(for_type.text.clone());
        Err(ResolveError::at(for_type, kind))
    }

    /// Records that this trait now has an instance for this type, refusing a second one.
    fn claimed(&mut self, of: &Name, for_type: &Name) -> Resolved {
        let claim = (of.text.clone(), for_type.text.clone());
        if self.instances.insert(claim) {
            return Ok(());
        }
        let kind = ResolveErrorKind::InstanceTwice {
            of: of.text.clone(),
            for_type: for_type.text.clone(),
        };
        Err(ResolveError::at(for_type, kind))
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
            Some(Origin::Prelude) => {
                Ok(named(&prelude::methods_of(&name.text).unwrap_or_default()))
            }
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

/// Refuses the first argument of an instance's type that is no type parameter it declares.
///
/// An instance is for one type, and a type written with arguments gets the one instance that
/// answers at every argument, so the arguments it is written with are the parameters it declares.
fn written_over_its_parameters(declaration: &InstanceDeclaration) -> Resolved {
    let declares = |written: &&Name| {
        declaration
            .type_parameters
            .iter()
            .any(|parameter| parameter.name.text == written.text)
    };
    let Some(other) = declaration.arguments.iter().find(|one| !declares(one)) else {
        return Ok(());
    };
    let kind = ResolveErrorKind::NotATypeParameter(other.text.clone());
    Err(ResolveError::at(other, kind))
}

/// Refuses a trait that is not one a type derives, naming the trait the module wrote.
fn derivable(named: &Name) -> Resolved {
    if prelude::DERIVABLE.contains(&named.text.as_str()) {
        return Ok(());
    }
    let kind = ResolveErrorKind::NotDerivable(named.text.clone());
    Err(ResolveError::at(named, kind))
}

/// The instances the prelude has, which a module writing one of them again is refused by.
pub(super) fn prelude_instances() -> HashSet<(String, String)> {
    prelude::instances()
        .map(|(of, for_type)| (of.to_owned(), for_type.to_owned()))
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
