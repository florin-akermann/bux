//! The `lumen` command-line interface.
//!
//! This crate is CLI only: argument parsing, file access, and help rendering. Compiler logic
//! lives in the per-phase crates under `crates/`, what canonical form is belongs to
//! `lumen-format`, and how a refusal reads belongs to `lumen-diagnostics`.

use std::ffi::OsStr;
use std::fs::{create_dir, create_dir_all, read_to_string, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{self, exit};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use lumen_diagnostics::{Code, Diagnostic, json, render};
use lumen_examples::{Example, Refusal as ExampleRefusal, Run, stated_by};
use lumen_format::format;
use lumen_holes::{Hole, Whole};
use lumen_ir::{Asked, Lowered, is_a_program, lower};
use lumen_jvm::ClassFile;
use lumen_types::TypedProgram;

use crate::compiling::{Checked, Module, NotCompiled, Refusal, checked, typed};

mod compiling;

/// The Lumen compiler.
#[derive(Parser)]
#[command(version, about, long_about = include_str!("help/lumen.md"))]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Rewrite a source file in canonical form
    #[command(long_about = include_str!("help/fmt.md"))]
    Fmt { file: PathBuf },
    /// Report the first thing about a source file the compiler will not have
    #[command(long_about = include_str!("help/check.md"))]
    Check {
        file: PathBuf,
        /// Write the refusal as one line of JSON on standard output, edit included
        #[arg(long)]
        json: bool,
    },
    /// Compile a source file to the class files a JVM loads
    #[command(long_about = include_str!("help/build.md"))]
    Build { file: PathBuf },
    /// Compile a source file and run the program it holds
    #[command(long_about = include_str!("help/run.md"))]
    Run { file: PathBuf },
    /// Run the examples a module states about its functions
    #[command(long_about = include_str!("help/test.md"))]
    Test { file: PathBuf },
    /// Print the public surface of a module
    #[command(long_about = include_str!("help/api.md"))]
    Api { file: PathBuf },
    /// Print the long form of one diagnostic code
    #[command(long_about = include_str!("help/explain.md"))]
    Explain { code: String },
}

fn main() {
    let Cli { command } = Cli::parse();
    exit(run(&command).code());
}

fn run(command: &Command) -> Outcome {
    match command {
        Command::Fmt { file } => fmt(file),
        Command::Check { file, json } => check(file, *json),
        Command::Build { file } => build(file),
        Command::Run { file } => started(file),
        Command::Test { file } => tested(file),
        Command::Api { file } => api(file),
        Command::Explain { code } => explain(code),
    }
}

/// Writes the canonical text of `path` back to it, leaving a canonical file untouched.
fn fmt(path: &Path) -> Outcome {
    let Some(source) = source_of(path) else {
        return Outcome::Unusable;
    };
    match format(&source) {
        Err(error) => refuse(&error.diagnostic(), &source, path),
        Ok(canonical) if canonical == source => Outcome::Done,
        Ok(canonical) => rewrite(path, &canonical),
    }
}

/// Reports the first thing about `path` the compiler will not have.
///
/// `as_data` asks for the refusal as JSON rather than as a page to read, which
/// `docs/specs/diagnostics.md` states field for field.
fn check(path: &Path, as_data: bool) -> Outcome {
    match checked(path) {
        Ok(_) => Outcome::Done,
        Err(NotCompiled::Unusable) => Outcome::Unusable,
        Err(NotCompiled::Refused(refusal)) if as_data => written_as_data(&refusal),
        Err(NotCompiled::Refused(refusal)) => shown(&refusal),
    }
}

/// Writes the refusal to standard output as data, because with `--json` that is what was asked for.
///
/// Checking stops at the first refusal, so there is the one to write.
fn written_as_data(refusal: &Refusal) -> Outcome {
    let first = refusal
        .diagnostics()
        .first()
        .expect("a refusal refuses something");
    print!("{}", json(first, &refusal.path().display().to_string()));
    Outcome::Refused
}

/// Prints every name `path` declares at the top level, with the type it has.
fn api(path: &Path) -> Outcome {
    match checked(path) {
        Ok(program) => {
            print!("{}", lumen_api::surface(program.root().typed()));
            Outcome::Done
        }
        Err(refusal) => refused_as(refusal),
    }
}

/// Runs every example the module in `path` states, reporting each that did not hold.
///
/// `docs/specs/doc-examples.md` says what an example is and what running one amounts to. A
/// module that states none has nothing to run, which is a run that held.
fn tested(path: &Path) -> Outcome {
    let program = match checked(path) {
        Ok(program) => program,
        Err(refusal) => return refused_as(refusal),
    };
    let root = program.root();
    match stated_in(root) {
        Err(refusals) => refuse_each(&refusals, root.source(), root.path()),
        Ok(None) => Outcome::Done,
        Ok(Some(run)) => held(&program, &run),
    }
}

/// The run that tries the examples `root` states, where it states any.
fn stated_in(root: &Module) -> Result<Option<Run>, Vec<Diagnostic>> {
    let source = root.source();
    let program = root.typed().resolved().program();
    let stated = stated_by(source, program).map_err(|refused| {
        refused
            .iter()
            .map(ExampleRefusal::diagnostic)
            .collect::<Vec<_>>()
    })?;
    if stated.is_empty() {
        return Ok(None);
    }
    let run =
        Run::of_module(source, program, stated).map_err(|refused| vec![refused.diagnostic()])?;
    Ok(Some(run))
}

/// Compiles the module `run` wrote, starts it, and reports every example that did not hold.
///
/// The class files go somewhere of the run's own, so nothing a build wrote is touched, and what
/// is written there is taken away again whether the examples held or not. Every module the one
/// under test imports is written there too, because the run reaches them as the module does.
fn held(program: &Checked, run: &Run) -> Outcome {
    let root = program.root();
    let Some(module) = named_module(root.path()) else {
        return Outcome::Unusable;
    };
    let lowered = match typed(run.source(), program.imported())
        .map_err(|diagnostic| vec![diagnostic])
        .and_then(|inferred| compiled(&inferred, run.source(), &module, &Asked::default()))
    {
        Ok(lowered) => lowered,
        Err(refusals) => {
            return refuse_each(&put_back(run, refusals), root.source(), root.path());
        }
    };
    let mut classes = match imported_classes(program, &lowered.asks) {
        Ok(classes) => classes,
        Err(refusal) => return refused_as(refusal),
    };
    classes.extend(lumen_jvm::write(&lowered));
    let Some(java) = java() else {
        return Outcome::Unusable;
    };
    let Some(beside) = somewhere_of_its_own() else {
        return Outcome::Unusable;
    };
    let outcome = match written(&classes, &beside) {
        Outcome::Done => match wrote(&java, &module, &beside) {
            Some(written) => did_not_hold(run, &written, root.source(), root.path()),
            None => Outcome::Unusable,
        },
        refusal => refusal,
    };
    drop(remove_dir_all(&beside));
    outcome
}

/// The class files of every module the one under test imports, which a run needs beside it.
///
/// `asked` is what the run itself asks of them, which is why the module under test is lowered
/// first: an example reaches a generic of an imported module as any other body does.
fn imported_classes(program: &Checked, asked: &Asked) -> Result<Vec<ClassFile>, NotCompiled> {
    let imported = program
        .modules()
        .split_last()
        .map_or(&[][..], |(_root, rest)| rest);
    let lowered = lowered_after_their_importers(imported, asked.clone())?;
    Ok(lowered.iter().flat_map(lumen_jvm::write).collect())
}

/// Reports every example the run wrote a line about, which is every one that did not hold.
fn did_not_hold(run: &Run, written: &str, source: &str, path: &Path) -> Outcome {
    let refusals: Vec<Diagnostic> = written
        .lines()
        .filter_map(|line| run.named(line))
        .map(Example::did_not_hold)
        .collect();
    if refusals.is_empty() {
        Outcome::Done
    } else {
        refuse_each(&refusals, source, path)
    }
}

/// Every refusal of the module a run wrote, said about the line in the file it was written from.
///
/// The run compiles like any module, so it is refused like any module, and a reader is owed the
/// line they wrote rather than the line the run wrote around it.
fn put_back(run: &Run, refusals: Vec<Diagnostic>) -> Vec<Diagnostic> {
    refusals
        .into_iter()
        .map(|diagnostic| {
            let written = diagnostic.span();
            diagnostic.about(run.in_original(written))
        })
        .collect()
}

/// What the module wrote to standard output, or nothing where it could not be run to the end.
fn wrote(java: &Path, module: &str, beside: &Path) -> Option<String> {
    match starting(java, module, beside).output() {
        Ok(ended) if ended.status.success() => String::from_utf8(ended.stdout).ok(),
        Ok(ended) => {
            eprint!("{}", String::from_utf8_lossy(&ended.stderr));
            None
        }
        Err(error) => {
            eprintln!("error: {}: {error}", java.display());
            None
        }
    }
}

/// Where a run writes the classes it is about to start, which is nowhere a build writes.
///
/// The directory is made here rather than found, and made so that a run never opens one that
/// was already there: the temporary directory is shared, and what a run writes into it is put
/// on a classpath and then deleted whole.
fn somewhere_of_its_own() -> Option<PathBuf> {
    let held = std::env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    for attempt in 0..ATTEMPTS_AT_A_DIRECTORY {
        let beside = held.join(format!("lumen-test-{}-{now:x}-{attempt}", process::id()));
        if create_dir(&beside).is_ok() {
            return Some(beside);
        }
    }
    eprintln!(
        "error: {}: nowhere to write the classes a run starts",
        held.display()
    );
    None
}

/// Writes the class files of `path` beside it, one per class the module becomes.
fn build(path: &Path) -> Outcome {
    built(path).map_or_else(|refusal| refusal, |_| Outcome::Done)
}

/// Builds `path` and runs the program it holds, which ends however that program ends.
///
/// The program is compiled before the JDK is looked for, so a program that does not compile is
/// told so on a machine that could not have run it anyway.
fn started(path: &Path) -> Outcome {
    let built = match built(path) {
        Ok(built) => built,
        Err(refusal) => return refusal,
    };
    if !built.starts {
        eprintln!(
            "error: {}: a module is run through `fn main() -> ()`, which this one does not declare",
            path.display()
        );
        return Outcome::Unusable;
    }
    let Some(java) = java() else {
        return Outcome::Unusable;
    };
    ran(&java, &built)
}

/// Compiles `path` and every module it reaches, writing the class files of each beside it.
///
/// A module is a class of its own, so a program of several modules is several classes, and a
/// JVM is started on the one the command named. `docs/specs/modules.md` states the order.
fn built(path: &Path) -> Result<Built, Outcome> {
    let program = checked(path).map_err(refused_as)?;
    let Some(module) = named_module(program.root().path()) else {
        return Err(Outcome::Unusable);
    };
    let lowered =
        lowered_after_their_importers(program.modules(), Asked::default()).map_err(refused_as)?;
    let starts = lowered.last().is_some_and(is_a_program);
    let classes: Vec<ClassFile> = lowered.iter().flat_map(lumen_jvm::write).collect();
    let beside = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    match written(&classes, &beside) {
        Outcome::Done => Ok(Built {
            module,
            beside,
            starts,
        }),
        refusal => Err(refusal),
    }
}

/// Every module of `modules`, lowered, in the order their classes are written.
///
/// A generic is written by the module that declares it, at each set of types a use settled it
/// at, which `docs/specs/codegen.md` states, and a use in one module asks the module it reaches
/// into. So a module is lowered after everything that imports it: loading orders them
/// dependencies first, and this runs that order backwards and turns the answer back around.
///
/// `asked` is what has been asked of them already, which is nothing for a build and what the
/// module under test asks for a run of its examples.
fn lowered_after_their_importers(
    modules: &[Module],
    asked: Asked,
) -> Result<Vec<Lowered>, NotCompiled> {
    let mut asked = asked;
    let mut lowered = Vec::new();
    for module in modules.iter().rev() {
        let one = lowered_from(module, &asked)?;
        asked = asked.and(&one.asks);
        lowered.push(one);
    }
    lowered.reverse();
    Ok(lowered)
}

/// The module `module` becomes, or every refusal that stops it becoming one.
fn lowered_from(module: &Module, asked: &Asked) -> Result<Lowered, NotCompiled> {
    compiled(module.typed(), module.source(), module.name(), asked)
        .map_err(|refusals| NotCompiled::refused(refusals, module.path(), module.source()))
}

/// The module `inferred` becomes, or every refusal that stops it becoming one.
///
/// Two things a build asks of a module that a check does not are asked here: every body is
/// written, which `docs/specs/holes.md` states, and every function says what it does, which
/// `docs/specs/doc-examples.md` states.
fn compiled(
    inferred: &TypedProgram,
    source: &str,
    module: &str,
    asked: &Asked,
) -> Result<Lowered, Vec<Diagnostic>> {
    let whole = Whole::of_module(inferred)
        .map_err(|holes| holes.iter().map(Hole::diagnostic).collect::<Vec<_>>())?;
    stated_by(source, inferred.resolved().program()).map_err(|refused| {
        refused
            .iter()
            .map(ExampleRefusal::diagnostic)
            .collect::<Vec<_>>()
    })?;
    Ok(lower(&whole, module, asked))
}

/// A module written out, which is what running one starts from.
struct Built {
    /// The class a JVM is started on, which is the module itself.
    module: String,
    /// Where the class files were written, which is where the JVM looks for them.
    beside: PathBuf,
    /// Whether the module declares `main`, which is what makes it a program.
    starts: bool,
}

/// How many names a run tries before it gives up on finding one nothing holds.
const ATTEMPTS_AT_A_DIRECTORY: u8 = 16;

/// What a class name cannot hold, because the JVM's internal form gives each of them a meaning.
const UNUSABLE_IN_A_NAME: [char; 4] = ['.', ';', '[', '/'];

/// The module `path` names, which is the name of the file it is written in.
///
/// A module class is named after the file, so a file whose name is not one a class may have
/// leaves nothing to write: a JVM would refuse to load what came out.
/// That is said here rather than returned, because there is nothing about the program to say.
fn named_module(path: &Path) -> Option<String> {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    if stem.is_empty() || stem.contains(UNUSABLE_IN_A_NAME) {
        eprintln!(
            "error: {}: a module is named by its file, and a class name holds none of {UNUSABLE_IN_A_NAME:?}",
            path.display()
        );
        return None;
    }
    Some(stem.into_owned())
}

/// Runs the module class on `java`, ending however the program it starts ends.
///
/// Every class written is a value class, which JDK 28 holds in preview, so the JVM is told to
/// load preview class files; `docs/implementation.md` section 1 says why.
fn ran(java: &Path, built: &Built) -> Outcome {
    match starting(java, &built.module, &built.beside).status() {
        Ok(status) => Outcome::Ended(status.code().unwrap_or(STOPPED)),
        Err(error) => {
            eprintln!("error: {}: {error}", java.display());
            Outcome::Unusable
        }
    }
}

/// A JVM told to start `module`, with the classes under `beside` and preview classes loadable.
fn starting(java: &Path, module: &str, beside: &Path) -> std::process::Command {
    let mut command = std::process::Command::new(java);
    command.args([
        OsStr::new("--enable-preview"),
        OsStr::new("-cp"),
        beside.as_os_str(),
    ]);
    command.arg(module);
    command
}

/// What a program stopped from outside is reported as, having ended with no status of its own.
///
/// It is neither of the statuses the compiler ends with, so a program the operating system
/// killed is never read as a program the compiler refused.
const STOPPED: i32 = 128;

/// The `java` of the JDK `JAVA_HOME` names, which is the only JVM a run ever reaches for.
fn java() -> Option<PathBuf> {
    let named = std::env::var_os("JAVA_HOME").filter(|home| !home.is_empty());
    let Some(home) = named else {
        eprintln!("error: JAVA_HOME is not set, and running a program needs the JDK it names");
        return None;
    };
    let java = Path::new(&home).join("bin").join("java");
    if !java.is_file() {
        eprintln!("error: {}: JAVA_HOME names no JDK", java.display());
        return None;
    }
    Some(java)
}

/// Writes each class beside the source file, in the package its name gives it.
fn written(classes: &[ClassFile], beside: &Path) -> Outcome {
    for class in classes {
        let written = beside.join(&class.path);
        if let Some(package) = written.parent()
            && let Err(error) = create_dir_all(package)
        {
            eprintln!("error: {}: {error}", package.display());
            return Outcome::Unusable;
        }
        if let Err(error) = write(&written, &class.bytes) {
            eprintln!("error: {}: {error}", written.display());
            return Outcome::Unusable;
        }
    }
    Outcome::Done
}

/// Prints what one diagnostic code means, at more length than its `help:` line has room for.
fn explain(written: &str) -> Outcome {
    let Some(code) = Code::written_as(written) else {
        eprintln!("error: there is no diagnostic {written}");
        return Outcome::Unusable;
    };
    print!("{}", code.explanation());
    Outcome::Done
}

/// What a run amounted to, which is what its exit code says.
enum Outcome {
    Done,
    /// The compiler will not have what it was given.
    Refused,
    /// The command could not do its job at all, which is not about any program.
    Unusable,
    /// A program ran, and this is the status it ended with rather than the compiler's.
    Ended(i32),
}

impl Outcome {
    const fn code(&self) -> i32 {
        match self {
            Self::Done => 0,
            Self::Refused => 1,
            Self::Unusable => 2,
            Self::Ended(status) => *status,
        }
    }
}

fn source_of(path: &Path) -> Option<String> {
    match read_to_string(path) {
        Ok(source) => Some(source),
        Err(error) => {
            eprintln!("error: {}: {error}", path.display());
            None
        }
    }
}

fn rewrite(path: &Path, canonical: &str) -> Outcome {
    match write(path, canonical) {
        Ok(()) => Outcome::Done,
        Err(error) => {
            eprintln!("error: {}: {error}", path.display());
            Outcome::Unusable
        }
    }
}

/// What a refusal of a module amounts to, having shown the reader every part of it.
fn refused_as(refusal: NotCompiled) -> Outcome {
    match refusal {
        NotCompiled::Unusable => Outcome::Unusable,
        NotCompiled::Refused(refusal) => shown(&refusal),
    }
}

/// Shows every refusal of one file, against the file it points into.
fn shown(refusal: &Refusal) -> Outcome {
    refuse_each(refusal.diagnostics(), refusal.source(), refusal.path())
}

/// Refuses every one rather than the first, because a build is how a reader learns what is left.
///
/// `docs/specs/holes.md` says why a hole is not stopped at the first of, and an example a module
/// does not state is not either. The blank line between two blocks is what keeps them two blocks.
fn refuse_each(refusals: &[Diagnostic], source: &str, path: &Path) -> Outcome {
    for (written, diagnostic) in refusals.iter().enumerate() {
        if written > 0 {
            eprintln!();
        }
        refuse(diagnostic, source, path);
    }
    Outcome::Refused
}

/// Reports why a file will not do, in the layout of `docs/specs/diagnostics.md`.
fn refuse(diagnostic: &Diagnostic, source: &str, path: &Path) -> Outcome {
    eprint!(
        "{}",
        render(diagnostic, source, &path.display().to_string())
    );
    Outcome::Refused
}
