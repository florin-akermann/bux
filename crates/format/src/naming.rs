//! How a declared name is spelled, which is part of canonical form and not of the text a printer
//! writes.
//!
//! `docs/design.md` section 13 spells a function in `snake_case` and a type in `PascalCase`, and
//! holds every declared name to a word rather than an initial. `docs/specs/naming.md` states the
//! rules. They are checked and never rewritten, for the same reason order is: what a thing is
//! called is the author's decision, so the compiler says what canonical form spells it rather than
//! spelling it.

use std::fmt;

use lumen_ast::{Function, Item, Name, Program, RecordField, Signature, Span};
use lumen_ast::{TypeDefinition, VariantPayload};
use lumen_diagnostics::Code;

/// The shortest a declared name is, below which it is an initial rather than a word.
const A_WORD: usize = 2;

/// The first declared name of `program` that canonical form spells differently, when there is one.
pub(crate) fn misspelled(program: &Program) -> Option<Misspelling> {
    declared(program)
        .into_iter()
        .find_map(|(name, kind)| Misspelling::of(name, kind))
}

/// A declared name canonical form does not spell the way the file wrote it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Misspelling {
    /// Written in neither `snake_case` nor `PascalCase`, or in the other one of the two.
    Case {
        written: String,
        kind: Kind,
        canonical: String,
        span: Span,
    },
    /// One character, which names nothing a reader can look for.
    Initial { written: String, span: Span },
}

impl Misspelling {
    /// The source the reader is pointed at, which is the name itself.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Case { span, .. } | Self::Initial { span, .. } => *span,
        }
    }

    /// The code this refusal carries, which says which of the two rules the name breaks.
    #[must_use]
    pub const fn code(&self) -> Code {
        match self {
            Self::Case { .. } => Code::NotCanonicalCase,
            Self::Initial { .. } => Code::NameIsAnInitial,
        }
    }

    /// What to do about it, which for a miscased name is the spelling canonical form gives it.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::Case { canonical, .. } => {
                format!("canonical form spells this name `{canonical}`")
            }
            Self::Initial { .. } => {
                "a declared name is a word, so write the one this names".to_owned()
            }
        }
    }

    /// How `name` departs from canonical form, when it does.
    ///
    /// The rule is about the word a name names, so the spelling canonical form gives it is what
    /// is measured rather than the characters the author happened to write. `_a` names one word
    /// of one letter, and reporting its case would advise a spelling that is itself an initial.
    fn of(name: &Name, kind: Kind) -> Option<Self> {
        if kind.may_be_an_initial() {
            return None;
        }
        let canonical = kind.spelling(&name.text);
        if canonical.chars().count() < A_WORD {
            return Some(Self::Initial {
                written: name.text.clone(),
                span: name.span,
            });
        }
        (canonical != name.text).then(|| Self::Case {
            written: name.text.clone(),
            kind,
            canonical,
            span: name.span,
        })
    }
}

impl fmt::Display for Misspelling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Case { written, kind, .. } => write!(
                f,
                "`{written}` is {}, so canonical form writes it in `{}`",
                kind.label(),
                kind.convention()
            ),
            Self::Initial { written, .. } => {
                write!(
                    f,
                    "`{written}` is an initial, which names nothing a reader can look for"
                )
            }
        }
    }
}

/// What a declared name names, which settles the case it is written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Function,
    Parameter,
    Field,
    Module,
    Type,
    Variant,
    TypeParameter,
}

impl Kind {
    /// How the message names this kind, which is what the reader is looking at.
    const fn label(self) -> &'static str {
        match self {
            Self::Function => "a function",
            Self::Parameter => "a parameter",
            Self::Field => "a record field",
            Self::Module => "a module",
            Self::Type => "a type",
            Self::Variant => "a variant",
            Self::TypeParameter => "a type parameter",
        }
    }

    /// The case canonical form writes this kind in, named as the message names it.
    const fn convention(self) -> &'static str {
        if self.writes_pascal_case() {
            "PascalCase"
        } else {
            "snake_case"
        }
    }

    /// Whether a name of this kind may be one character, which only a type parameter may be.
    ///
    /// A type parameter names no domain concept: it stands for whatever type a call supplies, and
    /// `T` is how that is written.
    const fn may_be_an_initial(self) -> bool {
        matches!(self, Self::TypeParameter)
    }

    /// The canonical spelling of `written` for this kind.
    fn spelling(self, written: &str) -> String {
        let words = words(written);
        if self.writes_pascal_case() {
            return words.iter().map(|word| capitalized(word)).collect();
        }
        words.join("_")
    }

    /// Whether this kind is a type rather than a value, which is what the two cases divide on.
    const fn writes_pascal_case(self) -> bool {
        matches!(self, Self::Type | Self::Variant | Self::TypeParameter)
    }
}

/// The words `written` is made of, each lowercased, split on `_` and on where the case turns.
///
/// An acronym is one word: `HTTPServer` is `http` and `server`, because the uppercase run ends
/// where the word that follows it begins. A digit continues the word it is written in, so `Int32`
/// is one word and not two.
fn words(written: &str) -> Vec<String> {
    let letters: Vec<char> = written.chars().collect();
    let mut words = Vec::new();
    let mut word = String::new();
    for (at, letter) in letters.iter().enumerate() {
        if *letter == '_' {
            push(&mut words, &mut word);
            continue;
        }
        if starts_a_word(&letters, at) {
            push(&mut words, &mut word);
        }
        word.extend(letter.to_lowercase());
    }
    push(&mut words, &mut word);
    words
}

/// Whether the character at `at` begins a word, which is where the case turns.
///
/// An uppercase letter after a lowercase one or a digit begins a word, as the `S` of `userService`
/// does. So does an uppercase letter that a lowercase one follows inside an uppercase run, as the
/// `S` of `HTTPServer` does, because the run before it is the word that ended.
fn starts_a_word(letters: &[char], at: usize) -> bool {
    if at == 0 || !letters[at].is_uppercase() {
        return false;
    }
    let before = letters[at - 1];
    !before.is_uppercase() && before != '_'
        || letters
            .get(at + 1)
            .is_some_and(|after| after.is_lowercase())
}

/// Keeps `word` when it holds anything, and empties it either way.
fn push(words: &mut Vec<String>, word: &mut String) {
    if !word.is_empty() {
        words.push(std::mem::take(word));
    }
}

/// `word` with its first character in uppercase, which is how `PascalCase` writes one.
fn capitalized(word: &str) -> String {
    let mut letters = word.chars();
    letters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(letters).collect()
    })
}

/// Every name `program` declares, with what it names, in the order the file writes them.
fn declared(program: &Program) -> Vec<(&Name, Kind)> {
    let mut names = Vec::new();
    for item in &program.items {
        match item {
            Item::Import(import) => names.push((&import.module, Kind::Module)),
            Item::Type(declared) => {
                names.push((&declared.name, Kind::Type));
                names.extend(
                    declared
                        .parameters
                        .iter()
                        .map(|one| (one, Kind::TypeParameter)),
                );
                definition(&mut names, &declared.definition);
            }
            Item::Trait(declared) => {
                names.push((&declared.name, Kind::Type));
                names.push((&declared.parameter, Kind::TypeParameter));
                for method in &declared.methods {
                    signature(&mut names, method);
                }
            }
            Item::Instance(declared) => {
                for method in &declared.methods {
                    function(&mut names, method);
                }
            }
            // A derive declares no name: it names a trait and a type both declared elsewhere.
            Item::Derive(_) => {}
            Item::Function(declared) => function(&mut names, declared),
        }
    }
    names
}

/// The names one function declares: itself, the types it is written over, and its parameters.
fn function<'a>(names: &mut Vec<(&'a Name, Kind)>, declared: &'a Function) {
    names.push((&declared.name, Kind::Function));
    names.extend(
        declared
            .type_parameters
            .iter()
            .map(|one| (&one.name, Kind::TypeParameter)),
    );
    names.extend(
        declared
            .parameters
            .iter()
            .map(|one| (&one.name, Kind::Parameter)),
    );
}

/// The names one method of a trait declares, which is itself and its parameters.
fn signature<'a>(names: &mut Vec<(&'a Name, Kind)>, declared: &'a Signature) {
    names.push((&declared.name, Kind::Function));
    names.extend(
        declared
            .parameters
            .iter()
            .map(|one| (&one.name, Kind::Parameter)),
    );
}

/// The names a type definition declares below its own, which are its variants and their fields.
fn definition<'a>(names: &mut Vec<(&'a Name, Kind)>, definition: &'a TypeDefinition) {
    match definition {
        TypeDefinition::Record(fields) => fields_of(names, fields),
        TypeDefinition::Variants(variants) => {
            for variant in variants {
                names.push((&variant.name, Kind::Variant));
                if let VariantPayload::Record(fields) = &variant.payload {
                    fields_of(names, fields);
                }
            }
        }
    }
}

/// The names of `fields`, each of them a record field.
fn fields_of<'a>(names: &mut Vec<(&'a Name, Kind)>, fields: &'a [RecordField]) {
    names.extend(fields.iter().map(|field| (&field.name, Kind::Field)));
}
