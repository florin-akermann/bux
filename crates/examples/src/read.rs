//! Reading the examples out of the comments a module writes.

use lumen_ast::{Function, Item, Program, Span, TypeRef, TypeRefKind};
use lumen_lexer::{TokenKind, lex};

use crate::example::Example;
use crate::refusal::Refusal;

/// What opens an example, after the comment marker and any blanks behind it.
const MARKER: &str = "example:";

/// The function a module declares at the top level that is run rather than called.
///
/// Running the module is the example of it, so there is no expression over it to state.
pub(crate) const REACHED_BY_RUNNING: &str = "main";

pub(crate) fn module(source: &str, program: &Program) -> Result<Vec<Example>, Vec<Refusal>> {
    let written = Commented::of(source);
    let mut stated: Vec<Example> = Vec::new();
    let mut refused = Vec::new();
    for function in documented(program) {
        let mut theirs = written.examples_of(function);
        if theirs.is_empty() {
            refused.push(Refusal::states_none(
                &function.name.text,
                function.name.span,
            ));
        }
        stated.append(&mut theirs);
    }
    refused.extend(written.documenting_nothing(&stated));
    refused.sort_by_key(|refusal| refusal.span().start());
    if refused.is_empty() {
        Ok(stated)
    } else {
        Err(refused)
    }
}

/// A module as the reading holds it: the text it is written in, and the comments it writes.
///
/// The two travel together because an example is a comment, and everything a comment states is
/// read out of the text its span covers.
pub(crate) struct Commented<'a> {
    source: &'a str,
    comments: Vec<Span>,
}

impl<'a> Commented<'a> {
    /// `source` with every comment it writes found, in source order.
    pub(crate) fn of(source: &'a str) -> Self {
        let comments = lex(source)
            .into_iter()
            .filter(|token| token.kind == TokenKind::LineComment)
            .map(|token| token.span)
            .collect();
        Self { source, comments }
    }

    /// The whole of one declaration as it is written: the comment above it, and the declaration.
    pub(crate) fn block_of(&self, declared: Span) -> Span {
        let opens = self
            .above(declared.start())
            .first()
            .map_or(declared.start(), |comment| self.opening_of(comment.start()));
        Span::new(opens, declared.end() - opens)
    }

    /// The examples the comment above `function` states about it.
    fn examples_of(&self, function: &Function) -> Vec<Example> {
        self.above(function.span.start())
            .into_iter()
            .filter_map(|at| {
                self.stated_at(at)
                    .map(|written| Example::stated(&function.name.text, written, at))
            })
            .collect()
    }

    /// Every example line that no function was read as stating.
    ///
    /// Inside a body, above a type, above `main`, and above nothing at all are each such a
    /// place. A line written there is one an author meant to state and no run would ever try.
    fn documenting_nothing(&self, stated: &[Example]) -> Vec<Refusal> {
        self.comments
            .iter()
            .filter(|&&comment| self.stated_at(comment).is_some())
            .filter(|&&comment| !stated.iter().any(|example| example.span() == comment))
            .map(|&comment| Refusal::documents_nothing(comment))
            .collect()
    }

    /// The comment lines written directly above the byte at `below`, in the order written.
    ///
    /// A comment joins the run when nothing but blanks precede it on its line and nothing but
    /// one line break separates it from what follows. A blank line therefore ends the run, and
    /// so does a comment written at the end of a line of code.
    fn above(&self, below: usize) -> Vec<Span> {
        let mut found = Vec::new();
        let mut reached = below;
        while let Some(&comment) = self
            .comments
            .iter()
            .rev()
            .find(|span| span.end() <= reached)
        {
            let opens = self.opening_of(comment.start());
            if !is_one_line_break(&self.source[comment.end()..reached])
                || !self.source[opens..comment.start()].chars().all(is_blank)
            {
                break;
            }
            found.push(comment);
            reached = opens;
        }
        found.reverse();
        found
    }

    /// The expression the comment at `at` states, where that comment is an example line.
    ///
    /// The marker is the run of slashes the comment opens with, so a line written `///` states
    /// an example as one written `//` does. A marker a run would pass over in silence is a
    /// claim nobody would ever check, which is the thing this whole page is against.
    fn stated_at(&self, at: Span) -> Option<&'a str> {
        let text = at.text(self.source).trim_start_matches('/');
        Some(text.trim_start().strip_prefix(MARKER)?.trim())
    }

    /// Where the line holding the byte at `at` begins.
    fn opening_of(&self, at: usize) -> usize {
        self.source[..at].rfind('\n').map_or(0, |ended| ended + 1)
    }
}

/// Every function the module declares at the top level that carries examples.
fn documented(program: &Program) -> impl Iterator<Item = &Function> {
    program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some(function),
            Item::Import(_)
            | Item::Type(_)
            | Item::Trait(_)
            | Item::Instance(_)
            | Item::Derive(_)
            | Item::Extern(_) => None,
        })
        .filter(|function| function.name.text != REACHED_BY_RUNNING)
        .filter(|function| !gives_nothing_back(function))
}

/// Whether the signature says the function gives nothing back, which leaves nothing to state.
///
/// An example is an expression that is true, and an expression over a call giving nothing back
/// is not one. `docs/specs/doc-examples.md` states it, and `main` is exempt for that same reason
/// as well as for being run rather than called.
fn gives_nothing_back(function: &Function) -> bool {
    matches!(
        function.result,
        Some(TypeRef {
            kind: TypeRefKind::Unit,
            ..
        })
    )
}

/// Whether `between` is one line break and nothing else that carries meaning.
fn is_one_line_break(between: &str) -> bool {
    between.matches('\n').count() == 1 && between.chars().all(|held| held == '\n' || is_blank(held))
}

/// Whether `held` separates tokens without being one, which `docs/specs/lexer.md` lists.
const fn is_blank(held: char) -> bool {
    matches!(held, ' ' | '\t' | '\r')
}
