//! The `lumen` command-line interface.
//!
//! This crate is CLI only: argument parsing, file access, and help rendering. Compiler logic
//! lives in the per-phase crates under `crates/`, what canonical form is belongs to
//! `lumen-format`, and how a refusal reads belongs to `lumen-diagnostics`.

use std::ffi::OsStr;
use std::fs::{create_dir, create_dir_all, read_dir, read_to_string, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{self, exit};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use lumen_diagnostics::{Code, Diagnostic, json, render};
use lumen_examples::{Example, Refusal as ExampleRefusal, Run, stated_by};
use lumen_format::format;
use lumen_ir::{THE_ONE_SHAPE, is_a_program};
use lumen_jvm::ClassFile;
use lumen_modules::{MANIFEST, SUFFIX};

use crate::compiling::{Checked, Module, NotCompiled, Refusal, checked};
use crate::lowering::lowered;

mod compiling;
mod lowering;

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
    /// Report the first thing about a source file or a package the compiler will not have
    #[command(long_about = concat!(include_str!("help/check.md"), include_str!("help/packages.md")))]
    Check {
        path: PathBuf,
        /// Write the refusal as one line of JSON on standard output, edit included
        #[arg(long)]
        json: bool,
    },
    /// Compile a source file or a package to the class files a JVM loads
    #[command(long_about = concat!(include_str!("help/build.md"), include_str!("help/packages.md")))]
    Build { path: PathBuf },
    /// Compile a source file and run the program it holds
    ///
    /// Every word after the file belongs to the program, `--help` among them, so this command
    /// has no help flag of its own; `lumen help run` is where its topic is read.
    #[command(long_about = include_str!("help/run.md"), disable_help_flag = true)]
    Run {
        file: PathBuf,
        /// The words the program is run with, which reach it as the list `main` takes
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        arguments: Vec<String>,
    },
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
        Command::Check { path, json } => check(path, *json),
        Command::Build { path } => build(path),
        Command::Run { file, arguments } => started(file, arguments),
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
/// A directory is a package, so every module of it is checked rather than one file, which
/// `docs/specs/packages.md` states. `as_data` asks for the refusal as JSON rather than as a page
/// to read, which `docs/specs/diagnostics.md` states field for field.
fn check(path: &Path, as_data: bool) -> Outcome {
    if path.is_dir() {
        return across(path, &|module| check(module, as_data));
    }
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
    let classes = match compiling::written(run.source(), root.path())
        .and_then(|written| lowered(written.modules()))
    {
        Ok(lowered) => lumen_jvm::write(&lowered),
        Err(NotCompiled::Unusable) => return Outcome::Unusable,
        Err(NotCompiled::Refused(refusal)) => return refusing(run, &refusal, root),
    };
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

/// A refusal of the run, shown against the file a reader would go and change.
///
/// A refusal of the run's own module points into the module the compiler wrote, so it is put
/// back into the file the author wrote first. One of a module that run imports points into that
/// module's own file already, and is shown there.
fn refusing(run: &Run, refusal: &Refusal, root: &Module) -> Outcome {
    if refusal.path() != root.path() {
        return refuse_each(refusal.diagnostics(), refusal.source(), refusal.path());
    }
    let refusals = put_back(run, refusal.diagnostics().to_vec());
    refuse_each(&refusals, root.source(), root.path())
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
///
/// The module is the one the run wrote, which reads no argument, so it is started with none.
fn wrote(java: &Path, module: &str, beside: &Path) -> Option<String> {
    match starting(java, module, beside, &[]).output() {
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
///
/// A directory is a package, so every module of it is built rather than one file, which
/// `docs/specs/packages.md` states.
fn build(path: &Path) -> Outcome {
    if path.is_dir() {
        return across(path, &build);
    }
    built(path).map_or_else(|refusal| refusal, |_| Outcome::Done)
}

/// Runs `over` on every module of the package in `directory`, stopping at the first refusal.
fn across(directory: &Path, over: &dyn Fn(&Path) -> Outcome) -> Outcome {
    let Some(modules) = modules_of(directory) else {
        return Outcome::Unusable;
    };
    modules
        .iter()
        .map(|module| over(module))
        .find(|outcome| !matches!(outcome, Outcome::Done))
        .unwrap_or(Outcome::Done)
}

/// Every module of the package in `directory`, in the order their names sort.
///
/// A directory holding no manifest is no package, and a directory that cannot be listed holds
/// nothing to compile. Each is said here rather than returned, because neither is about a
/// program: there is no manifest for a refusal to point into and no source to point at.
fn modules_of(directory: &Path) -> Option<Vec<PathBuf>> {
    if !directory.join(MANIFEST).is_file() {
        eprintln!(
            "error: {}: there is no package here, which is a directory holding `{MANIFEST}`",
            directory.display()
        );
        return None;
    }
    let listed = match read_dir(directory) {
        Ok(listed) => listed,
        Err(error) => {
            eprintln!("error: {}: {error}", directory.display());
            return None;
        }
    };
    let mut modules: Vec<PathBuf> = listed
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|written| written == SUFFIX))
        .collect();
    modules.sort();
    Some(modules)
}

/// Builds `path` and runs the program it holds, which ends however that program ends.
///
/// The program is compiled before the JDK is looked for, so a program that does not compile is
/// told so on a machine that could not have run it anyway. `arguments` is every word written
/// after the file, which reaches the program and nothing else.
fn started(path: &Path, arguments: &[String]) -> Outcome {
    let built = match built(path) {
        Ok(built) => built,
        Err(refusal) => return refusal,
    };
    if !built.starts {
        eprintln!(
            "error: {}: a module is run through `{THE_ONE_SHAPE}`, which this one does not declare",
            path.display()
        );
        return Outcome::Unusable;
    }
    let Some(java) = java() else {
        return Outcome::Unusable;
    };
    ran(&java, &built, arguments)
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
    let lowered = lowered(program.modules()).map_err(refused_as)?;
    let starts = lowered.last().is_some_and(is_a_program);
    let classes = lumen_jvm::write(&lowered);
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
fn ran(java: &Path, built: &Built, arguments: &[String]) -> Outcome {
    match starting(java, &built.module, &built.beside, arguments).status() {
        Ok(status) => Outcome::Ended(status.code().unwrap_or(STOPPED)),
        Err(error) => {
            eprintln!("error: {}: {error}", java.display());
            Outcome::Unusable
        }
    }
}

/// A JVM told to start `module`, with the classes under `beside` and preview classes loadable.
///
/// Every word of `arguments` goes to the program unchanged and in order, after the class the
/// JVM starts on, so nothing the runner writes is ever read as one of them.
fn starting(
    java: &Path,
    module: &str,
    beside: &Path,
    arguments: &[String],
) -> std::process::Command {
    let mut command = std::process::Command::new(java);
    command.args([
        OsStr::new("--enable-preview"),
        OsStr::new("-cp"),
        beside.as_os_str(),
    ]);
    command.arg(module);
    command.args(arguments);
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
