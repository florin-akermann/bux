//! The resolver's printed form: each resolved name on a line of its own, or the refusal.
//!
//! `docs/specs/modules.md` states the form. The resolver's tests read it, and so does the harness
//! of the resolver written in Bux, `crates/cli/tests/integration/bux_resolver.rs`, which includes
//! this file by its path.

use std::fmt::Write as _;

use lumen_ast::Span;
use lumen_resolver::{
    Definition, DefinitionKind, Namespace, Origin, ResolveError, ResolvedProgram,
};
use lumen_resolver::{prelude_resolved, resolve};

/// The name a fixture is resolved as, which nothing that is printed depends on.
const MODULE: &str = "main";

/// What the resolver says of `source`: the resolved names, the refusal, or `unparsed`.
pub fn render(source: &str) -> String {
    let Ok(program) = lumen_parser::parse(source) else {
        return "unparsed\n".to_owned();
    };
    match resolve(program, MODULE) {
        Ok(resolved) => render_resolved(&resolved),
        Err(error) => render_refusal(&error),
    }
}

/// What the resolver says of the prelude, which is resolved with the names the JVM holds.
pub fn render_prelude() -> String {
    render_resolved(prelude_resolved())
}

/// `resolved`, then each name that has a definition, in the order of the spans.
fn render_resolved(resolved: &ResolvedProgram) -> String {
    let mut lines: Vec<(Namespace, Span, Definition)> = resolved.resolutions().collect();
    lines.sort_by_key(|(namespace, span, _)| (span.start(), namespace_word(*namespace)));
    let mut written = "resolved\n".to_owned();
    for (namespace, span, definition) in lines {
        let _ = writeln!(
            written,
            "{} {} {} {}",
            shown(span),
            namespace_word(namespace),
            kind_word(definition.kind),
            origin_words(definition.origin)
        );
    }
    written
}

/// `refused`, then the code, the span, and the message, then the help.
fn render_refusal(error: &ResolveError) -> String {
    let said = error.diagnostic();
    format!(
        "refused\n{} {} {}\nhelp: {}\n",
        said.code().number(),
        shown(said.span()),
        said.message(),
        error.help()
    )
}

fn shown(span: Span) -> String {
    format!("{}..{}", span.start(), span.end())
}

/// The word of a namespace, which also sorts a `type` line above a `value` line at one span.
const fn namespace_word(namespace: Namespace) -> &'static str {
    match namespace {
        Namespace::Type => "type",
        Namespace::Value => "value",
    }
}

const fn kind_word(kind: DefinitionKind) -> &'static str {
    match kind {
        DefinitionKind::Module => "module",
        DefinitionKind::Type => "type",
        DefinitionKind::Trait => "trait",
        DefinitionKind::TypeParameter => "type-parameter",
        DefinitionKind::Constructor => "constructor",
        DefinitionKind::Function => "function",
        DefinitionKind::TraitMethod => "trait-method",
        DefinitionKind::Parameter => "parameter",
        DefinitionKind::Local => "local",
        DefinitionKind::Variable => "variable",
    }
}

fn origin_words(origin: Origin) -> String {
    match origin {
        Origin::Prelude => "prelude".to_owned(),
        Origin::Declared(at) => format!("declared {}", shown(at)),
    }
}
