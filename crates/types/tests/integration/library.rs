//! `docs/specs/library.md`: the prelude is compiled with every check a program is compiled with.
//!
//! The prelude is Lumen source the compiler carries, and its bodies say in Lumen what the JVM
//! instructions do. A body that said something else would be a claim nothing held, so inference
//! walks them exactly as it walks a module's, and these are what that walk is held to.

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ast::{Function, InstanceDeclaration, Item, Signature, Span, StatementKind};
use lumen_ast::{TraitDeclaration, TypeRef, TypeRefKind};
use lumen_resolver::prelude_resolved;
use lumen_types::{Imported, TypedProgram, check};

#[test]
fn the_prelude_the_compiler_carries_is_a_module_that_compiles() {
    check(prelude_resolved().clone(), &Imported::default())
        .expect("every body the prelude writes has the type its declaration gives it");
}

#[test]
fn a_bool_parameter_a_trait_gave_an_instance_is_no_flag_the_prelude_could_have_avoided() {
    let typed = prelude_typed();
    let method = named("Hash<Bool>.hashed");

    assert_eq!(signature_found(&typed, &method), "(Bool) -> Int");
    assert_eq!(value_found(&typed, &method), "Int");
    assert_eq!(method.declared, "(Bool) -> Int");
}

/// `docs/specs/library.md`: every instance body the prelude writes has the type its trait gives it.
///
/// The trait says what the method is at the type the instance is for, and inference says what the
/// declaration and the body each turned out to be. The property is that reading any of the three
/// never turns up anything the other two disagree with.
///
/// The body is the half a signature alone would not show: its type is one inference worked out
/// from what the body does, so a body nothing walked has no type here at all.
#[hegel::test]
fn every_instance_body_the_prelude_writes_has_the_type_its_trait_gives_it(tc: TestCase) {
    let typed = prelude_typed();
    let method = tc.draw(gs::sampled_from(every_instance_method()));

    assert_eq!(
        signature_found(&typed, &method),
        method.declared,
        "{}",
        method.reads_as
    );
    assert_eq!(
        value_found(&typed, &method),
        gives(&method.declared),
        "{}",
        method.reads_as
    );
}

/// One method one instance of the prelude writes: where it is, and what its trait says.
#[derive(Clone, Debug)]
struct Method {
    /// Where the instance writes its name, which is where inference recorded its signature.
    written: Span,
    /// Where the body's value is written, which is where inference recorded what it came to.
    leaves: Span,
    /// The type its trait gives it, at the type the instance is for.
    declared: String,
    /// `Hash<Bool>.hashed`: what a failure names, and what a test asks for one of them by.
    reads_as: String,
}

/// The prelude, inferred as a module is inferred, which is what its bodies are read by.
fn prelude_typed() -> TypedProgram {
    check(prelude_resolved().clone(), &Imported::default())
        .expect("the prelude the compiler carries is a module that compiles")
}

/// The signature inference gave `method`, read where the instance writes its name.
fn signature_found(typed: &TypedProgram, method: &Method) -> String {
    at(typed, method.written)
}

/// The type inference gave what `method`'s body leaves behind, read from the body itself.
fn value_found(typed: &TypedProgram, method: &Method) -> String {
    at(typed, method.leaves)
}

/// The type inference recorded at `written`, which it has for everything it walked.
fn at(typed: &TypedProgram, written: Span) -> String {
    typed
        .type_of(written)
        .expect("inference gave everything it walked a type")
        .to_string()
}

/// What a signature written as text gives back, which is what its body has to leave behind.
fn gives(signature: &str) -> &str {
    signature
        .split_once("-> ")
        .expect("a signature as text is written with an arrow")
        .1
}

/// The one method of the prelude that `reads_as` names.
fn named(reads_as: &str) -> Method {
    every_instance_method()
        .into_iter()
        .find(|one| one.reads_as == reads_as)
        .unwrap_or_else(|| panic!("the prelude writes `{reads_as}`"))
}

/// Every method every instance of the prelude writes, in the order the source writes them.
fn every_instance_method() -> Vec<Method> {
    let items = &prelude_resolved().program().items;
    instances(items)
        .flat_map(|of| of.methods.iter().map(|method| read(items, of, method)))
        .collect()
}

/// One instance method: where its body is, what its trait gives it, and what it reads as.
fn read(items: &[Item], of: &InstanceDeclaration, method: &Function) -> Method {
    let declaration = declares(items, of);
    let standing = StandingIn {
        parameter: &declaration.parameter.text,
        instance: written_over(of),
    };
    Method {
        written: method.name.span,
        leaves: leaves(method),
        declared: at_the_type(signature(declaration, method), &standing),
        reads_as: format!(
            "{}<{}>.{}",
            of.trait_name.text, of.for_type.text, method.name.text
        ),
    }
}

/// The type the instance is for, as it writes it: its name, and the arguments it is written with.
fn written_over(of: &InstanceDeclaration) -> String {
    if of.arguments.is_empty() {
        return of.for_type.text.clone();
    }
    let written: Vec<&str> = of.arguments.iter().map(|one| one.text.as_str()).collect();
    format!("{}<{}>", of.for_type.text, written.join(", "))
}

/// Where the value `method` gives back is written, which is its body's last statement.
///
/// Every method the prelude writes is one expression, so the body's value is that expression and
/// the type inference worked out for it is what the body came to.
fn leaves(method: &Function) -> Span {
    let last = method
        .body
        .statements
        .last()
        .expect("a method of the prelude writes a body");
    match &last.kind {
        StatementKind::Expr(value) => value.span,
        _ => unreachable!("a method of the prelude leaves an expression behind"),
    }
}

/// The trait's own parameter, and the type the instance settled it on.
struct StandingIn<'p> {
    parameter: &'p str,
    /// The type as the instance writes it, which is `List<T>` where it writes arguments.
    instance: String,
}

/// The trait `of` answers for, which the prelude declares below it.
fn declares<'p>(items: &'p [Item], of: &InstanceDeclaration) -> &'p TraitDeclaration {
    traits(items)
        .find(|one| one.name.text == of.trait_name.text)
        .expect("an instance answers for a trait the prelude declares")
}

/// The signature `declaration` gives the method `method` answers for.
fn signature<'p>(declaration: &'p TraitDeclaration, method: &Function) -> &'p Signature {
    declaration
        .methods
        .iter()
        .find(|one| one.name.text == method.name.text)
        .expect("an instance writes a method its trait declares")
}

/// One trait method as text, with its trait's own parameter standing for the instance's type.
fn at_the_type(signature: &Signature, standing: &StandingIn<'_>) -> String {
    let taken: Vec<String> = signature
        .parameters
        .iter()
        .map(|one| {
            let written = one
                .type_ref
                .as_ref()
                .expect("a trait writes the type of every parameter it declares");
            standing_for(written, standing)
        })
        .collect();
    let gives = signature
        .result
        .as_ref()
        .map_or_else(|| "()".to_owned(), |one| standing_for(one, standing));
    format!("({}) -> {gives}", taken.join(", "))
}

/// One written type as text, with the trait's parameter standing for the instance's type.
fn standing_for(written: &TypeRef, standing: &StandingIn<'_>) -> String {
    let TypeRefKind::Named { path, arguments } = &written.kind else {
        return "()".to_owned(); // `TypeRefKind::Unit`, the only other shape a written type takes.
    };
    let name = if path.name.text == standing.parameter {
        &standing.instance
    } else {
        &path.name.text
    };
    if arguments.is_empty() {
        return name.to_owned();
    }
    let applied: Vec<String> = arguments
        .iter()
        .map(|one| standing_for(one, standing))
        .collect();
    format!("{name}<{}>", applied.join(", "))
}

/// Every instance the prelude writes, in the order it writes them.
fn instances(items: &[Item]) -> impl Iterator<Item = &InstanceDeclaration> {
    items.iter().filter_map(|item| match item {
        Item::Instance(declaration) => Some(declaration),
        _ => None,
    })
}

/// Every trait the prelude declares, in the order it declares them.
fn traits(items: &[Item]) -> impl Iterator<Item = &TraitDeclaration> {
    items.iter().filter_map(|item| match item {
        Item::Trait(declaration) => Some(declaration),
        _ => None,
    })
}
