//! A function as the page states it: its name, what it takes, and what it gives back.

use lumen_ast::{Function, Name, TypeParameter};
use lumen_types::{Type, TypedProgram};

/// `fn shared(total: Int, people: Int) -> Int`, and the newline that ends the line.
///
/// The types are the ones inference settled on, so a parameter the author left unwritten reads
/// the same as one they wrote out.
pub(crate) fn of(typed: &TypedProgram, declared: &Function) -> String {
    let Type::Function { parameters, result } = whole(typed, &declared.name) else {
        unreachable!("a function is declared with a function type")
    };
    let takes: Vec<String> = declared
        .parameters
        .iter()
        .zip(parameters)
        .map(|(taken, has)| format!("{}: {has}", taken.name.text))
        .collect();
    format!(
        "fn {}{}({}) -> {result}\n",
        declared.name.text,
        over(&declared.type_parameters),
        takes.join(", ")
    )
}

/// The type inference gave the function declared at `name`.
fn whole(typed: &TypedProgram, name: &Name) -> Type {
    typed
        .type_of(name.span)
        .expect("inference gives every declared function a type")
        .clone()
}

/// `<T, E>` or `<T: Eq<T>>`, which most functions do not have.
fn over(written: &[TypeParameter]) -> String {
    if written.is_empty() {
        return String::new();
    }
    let each: Vec<String> = written.iter().map(constrained).collect();
    format!("<{}>", each.join(", "))
}

/// One type parameter as the page states it, with each trait it is constrained by.
fn constrained(written: &TypeParameter) -> String {
    if written.constraints.is_empty() {
        return written.name.text.clone();
    }
    let each: Vec<String> = written
        .constraints
        .iter()
        .map(|constraint| {
            format!(
                "{}<{}>",
                constraint.name.text,
                lumen_format::written_type(&constraint.argument)
            )
        })
        .collect();
    format!("{}: {}", written.name.text, each.join(" + "))
}
