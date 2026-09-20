//! The public surface of a module, printed from the types inference gave it.
//!
//! The crate consumes the typed tree and yields one page of text: every name the module declares
//! at the top level, with the type it has. `docs/specs/api-surface.md` is the specification.

mod signature;

use lumen_ast::Item;
use lumen_types::TypedProgram;

/// The public surface of `typed`, as one page of text.
///
/// The page is empty when the module declares nothing, and otherwise ends with a newline.
#[must_use]
pub fn surface(typed: &TypedProgram) -> String {
    let mut page = String::new();
    for declared in &typed.resolved().program().items {
        let Some(stanza) = stanza(typed, declared) else {
            continue;
        };
        if !page.is_empty() {
            page.push('\n');
        }
        page.push_str(&stanza);
    }
    page
}

/// What one item puts on the page, which is nothing at all for an import.
///
/// An import brings a name in rather than putting one out, so a module's surface never holds one.
fn stanza(typed: &TypedProgram, declared: &Item) -> Option<String> {
    match declared {
        Item::Import(_) => None,
        Item::Type(declaration) => Some(lumen_format::type_declaration(declaration)),
        Item::Trait(declaration) => Some(lumen_format::trait_declaration(declaration)),
        Item::Instance(declaration) => Some(lumen_format::instance_head(declaration)),
        Item::Derive(declaration) => Some(lumen_format::derived_head(declaration)),
        Item::Function(declaration) => Some(signature::of(typed, declaration)),
    }
}
