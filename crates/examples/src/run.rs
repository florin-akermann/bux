//! The module a run writes to try the examples a module states.

use std::fmt::Write as _;

use lumen_ast::{Item, Program, Span};

use crate::example::Example;
use crate::read;
use crate::refusal::Refusal;

/// The module a run reaches for to say which example did not hold.
const WRITES_A_LINE: &str = "io";

/// The signature a JVM starts a module on, which the run writes for itself.
const ENTRY: &str = "fn main(arguments: List<String>) -> Int";

/// The name `ENTRY` takes its arguments under, which the module under test may not also declare.
///
/// A module declaring the same name at the top level would put two of it in scope, so a run says
/// so rather than writing a module the author cannot read a refusal of.
const TAKES_THE_ARGUMENTS: &str = "arguments";

/// What the written `main` ends with, which is the status a run that reached the end gives back.
///
/// The run says which example did not hold by writing a line, so the status says nothing more
/// than that the run reached the end.
const ENDS: &str = "    0\n}\n";

/// What a line the run wrote opens with, which is what tells it from the program's own output.
///
/// An example is free to write a line of its own, and `io.println` is the one channel there is.
/// A run reading bare ordinals back would read `io.println("0")` inside an example as the first
/// example saying it did not hold. Every line the run writes is marked, and a line without the
/// mark is the program talking.
const SAYS_ONE_DID_NOT_HOLD: &str = "lumen: example ";

/// The module `lumen test` runs, and where each stretch of it points in the file it came from.
///
/// A module is the only thing there is to run, so the examples are run as one: the imports, then
/// a `main` that tries each example in turn, then every declaration the original makes other than
/// its own `main`. Each example is tried by an `if` that writes the ordinal of the example when
/// it is not `true`, so what the run reads back is exactly the examples that did not hold.
#[derive(Debug)]
pub struct Run {
    source: String,
    stretches: Vec<Stretch>,
    examples: Vec<Example>,
    module: Span,
}

impl Run {
    /// The module that tries every example in `stated`, written out of `source`.
    ///
    /// # Errors
    ///
    /// Returns the declaration of a name the run writes with where the module makes one, because
    /// a module declares a name once. The run reaches `io` to say which example did not hold,
    /// and the `main` it writes takes its arguments under a name of its own.
    pub fn of_module(
        source: &str,
        program: &Program,
        stated: Vec<Example>,
    ) -> Result<Self, Refusal> {
        for named in [WRITES_A_LINE, TAKES_THE_ARGUMENTS] {
            if let Some(declared) = declaring(program, named) {
                return Err(Refusal::reaches_for(named, declared));
            }
        }
        let mut written = Written::default();
        written.wrote(&imports_of(program));
        written.wrote(&format!("{ENTRY} {{\n"));
        for (ordinal, example) in stated.iter().enumerate() {
            written.around(ordinal, &tried(example, ordinal));
        }
        written.wrote(ENDS);
        for declaration in declarations_of(source, program) {
            written.wrote("\n");
            written.copied(source, declaration);
            written.wrote("\n");
        }
        Ok(Self {
            source: written.source,
            stretches: written.stretches,
            examples: stated,
            module: Span::new(0, source.len()),
        })
    }

    /// What every line the run writes opens with, and nothing the program writes is expected to.
    #[must_use]
    pub const fn marks() -> &'static str {
        SAYS_ONE_DID_NOT_HOLD
    }

    /// The module the run wrote, which is what is compiled and started.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The example one line the run wrote names, where that line is the run's and names one.
    #[must_use]
    pub fn named(&self, line: &str) -> Option<&Example> {
        let ordinal = line.trim_end().strip_prefix(SAYS_ONE_DID_NOT_HOLD)?;
        self.examples.get(ordinal.parse::<usize>().ok()?)
    }

    /// Where the text at `written` points in the file the examples were read out of.
    ///
    /// The run is compiled like any other module, so it can be refused like any other, and a
    /// reader is owed the line they wrote rather than the line the run wrote around it.
    #[must_use]
    pub fn in_original(&self, written: Span) -> Span {
        let Some(stretch) = self.holding(written) else {
            return self.module;
        };
        match stretch.about {
            About::SameText { from } => {
                let into = written.start() - stretch.written.start();
                Span::new(
                    from + into,
                    written.bytes().min(stretch.written.bytes() - into),
                )
            }
            About::OneExample(ordinal) => self.examples[ordinal].span(),
            About::WholeModule => self.module,
        }
    }

    fn holding(&self, written: Span) -> Option<&Stretch> {
        self.stretches
            .iter()
            .find(|stretch| stretch.written.range().contains(&written.start()))
    }
}

/// One stretch of a written module: where it lies in it, and what a span inside it is about.
#[derive(Debug)]
struct Stretch {
    written: Span,
    about: About,
}

/// What a span inside one stretch of a written module is about.
#[derive(Clone, Copy, Debug)]
enum About {
    /// Source copied byte for byte, which a span inside is about one shift away.
    SameText { from: usize },
    /// Text written around one example, which a span inside is about that example.
    OneExample(usize),
    /// Text the run wrote for itself, which a span inside is about the module as a whole.
    WholeModule,
}

/// A module being written, with a note of where each stretch of it came from.
#[derive(Default)]
struct Written {
    source: String,
    stretches: Vec<Stretch>,
}

impl Written {
    /// Writes text the run made up for itself.
    fn wrote(&mut self, text: &str) {
        self.holding(text, About::WholeModule);
    }

    /// Writes the text the run made up around one example.
    fn around(&mut self, ordinal: usize, text: &str) {
        self.holding(text, About::OneExample(ordinal));
    }

    /// Writes the stretch of `source` that `from` names, byte for byte.
    fn copied(&mut self, source: &str, from: Span) {
        self.holding(from.text(source), About::SameText { from: from.start() });
    }

    fn holding(&mut self, text: &str, about: About) {
        self.stretches.push(Stretch {
            written: Span::new(self.source.len(), text.len()),
            about,
        });
        self.source.push_str(text);
    }
}

/// The `if` that tries one example and writes its ordinal where it did not hold.
fn tried(example: &Example, ordinal: usize) -> String {
    let written = example.expression();
    let says = format!("{SAYS_ONE_DID_NOT_HOLD}{ordinal}");
    format!("    if !({written}) {{\n        {WRITES_A_LINE}.println(\"{says}\")\n    }}\n")
}

/// Where the module declares `named` at the top level, and nowhere where it declares no such name.
fn declaring(program: &Program, named: &str) -> Option<Span> {
    program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Import(_) | Item::Instance(_) | Item::Derive(_) => None,
            Item::Type(declaration) => Some(&declaration.name),
            Item::Trait(declaration) => Some(&declaration.name),
            Item::Function(function) => Some(&function.name),
            Item::Extern(declaration) => Some(&declaration.name),
        })
        .find(|declared| declared.text == named)
        .map(|declared| declared.span)
}

/// The imports the written module opens with: the module's own, and the one the run writes with.
fn imports_of(program: &Program) -> String {
    let mut modules: Vec<&str> = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Import(import) => Some(import.module.text.as_str()),
            Item::Type(_)
            | Item::Trait(_)
            | Item::Instance(_)
            | Item::Derive(_)
            | Item::Function(_)
            | Item::Extern(_) => None,
        })
        .collect();
    modules.push(WRITES_A_LINE);
    modules.sort_unstable();
    modules.dedup();
    modules.iter().fold(String::new(), |mut written, module| {
        let _ = writeln!(written, "import {module}\n");
        written
    })
}

/// Every declaration the module makes other than its own `main`, comment and all.
///
/// The module's `main` is left out because the examples are what a run runs; a module is put
/// through `lumen run` to run the program it holds.
fn declarations_of(source: &str, program: &Program) -> Vec<Span> {
    let written = read::Commented::of(source);
    program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Import(_) => None,
            Item::Type(declaration) => Some(declaration.span),
            Item::Trait(declaration) => Some(declaration.span),
            Item::Instance(declaration) => Some(declaration.span),
            Item::Derive(declaration) => Some(declaration.span),
            Item::Extern(declaration) => Some(declaration.span),
            Item::Function(function) if function.name.text == read::REACHED_BY_RUNNING => None,
            Item::Function(function) => Some(function.span),
        })
        .map(|declared| written.block_of(declared))
        .collect()
}
