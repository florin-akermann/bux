//! The `lumen` command-line interface.
//!
//! This crate is CLI only: argument parsing, file access, and help rendering. Compiler logic
//! lives in the per-phase crates under `crates/`, what canonical form is belongs to
//! `lumen-format`, and how a refusal reads belongs to `lumen-diagnostics`.

use std::ffi::OsStr;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{Parser, Subcommand};
use lumen_diagnostics::{Code, Diagnostic, render};
use lumen_exhaustiveness::check as exhaustive;
use lumen_format::format;
use lumen_holes::{Hole, Whole};
use lumen_ir::{is_a_program, lower};
use lumen_jvm::ClassFile;
use lumen_parser::parse;
use lumen_resolver::resolve;
use lumen_types::{TypedProgram, check as inferred};

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
    Check { file: PathBuf },
    /// Compile a source file to the class files a JVM loads
    #[command(long_about = include_str!("help/build.md"))]
    Build { file: PathBuf },
    /// Compile a source file and run the program it holds
    #[command(long_about = include_str!("help/run.md"))]
    Run { file: PathBuf },
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
        Command::Check { file } => check(file),
        Command::Build { file } => build(file),
        Command::Run { file } => started(file),
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
fn check(path: &Path) -> Outcome {
    let Some(source) = source_of(path) else {
        return Outcome::Unusable;
    };
    match accepted(&source) {
        Ok(_) => Outcome::Done,
        Err(diagnostic) => refuse(&diagnostic, &source, path),
    }
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

/// Compiles `path` and writes the class files beside it, saying what a JVM would start on.
fn built(path: &Path) -> Result<Built, Outcome> {
    let Some(source) = source_of(path) else {
        return Err(Outcome::Unusable);
    };
    let typed = match accepted(&source) {
        Ok(typed) => typed,
        Err(diagnostic) => return Err(refuse(&diagnostic, &source, path)),
    };
    let whole = match Whole::of_module(&typed) {
        Ok(whole) => whole,
        Err(holes) => return Err(refuse_each(&holes, &source, path)),
    };
    let Some(module) = module_of(path) else {
        eprintln!(
            "error: {}: a module is named by its file, and a class name holds none of {UNUSABLE_IN_A_NAME:?}",
            path.display()
        );
        return Err(Outcome::Unusable);
    };
    let lowered = lower(&whole, &module);
    match written(&lumen_jvm::write(&lowered), path) {
        Outcome::Done => Ok(Built {
            module,
            beside: path.parent().unwrap_or(Path::new(".")).to_path_buf(),
            starts: is_a_program(&lowered),
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

/// Every phase the front end has, run in order, stopping at the first refusal.
fn accepted(source: &str) -> Result<TypedProgram, Diagnostic> {
    lumen_format::check(source).map_err(|error| error.diagnostic())?;
    let program = parse(source).map_err(|error| error.diagnostic())?;
    let resolved = resolve(program).map_err(|error| error.diagnostic())?;
    let typed = inferred(resolved).map_err(|error| error.diagnostic())?;
    exhaustive(&typed).map_err(|error| error.diagnostic())?;
    Ok(typed)
}

/// What a class name cannot hold, because the JVM's internal form gives each of them a meaning.
const UNUSABLE_IN_A_NAME: [char; 4] = ['.', ';', '[', '/'];

/// The name the module takes, which is the name of the file it is written in.
///
/// A module class is named after the file, so a file whose name is not one a class may have
/// leaves nothing to write: a JVM would refuse to load what came out.
fn module_of(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let usable = !stem.is_empty() && !stem.contains(UNUSABLE_IN_A_NAME);
    usable.then_some(stem)
}

/// Runs the module class on `java`, ending however the program it starts ends.
///
/// Every class written is a value class, which JDK 28 holds in preview, so the JVM is told to
/// load preview class files; `docs/implementation.md` section 1 says why.
fn ran(java: &Path, built: &Built) -> Outcome {
    let arguments = [
        OsStr::new("--enable-preview"),
        OsStr::new("-cp"),
        built.beside.as_os_str(),
    ];
    match std::process::Command::new(java)
        .args(arguments)
        .arg(&built.module)
        .status()
    {
        Ok(status) => Outcome::Ended(status.code().unwrap_or(STOPPED)),
        Err(error) => {
            eprintln!("error: {}: {error}", java.display());
            Outcome::Unusable
        }
    }
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
fn written(classes: &[ClassFile], path: &Path) -> Outcome {
    let beside = path.parent().unwrap_or(Path::new("."));
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

/// Refuses every hole rather than the first, because a build is how a reader learns what is left.
///
/// `docs/specs/holes.md` says why this is the one refusal that does not stop at the first, and
/// the blank line between two blocks is what keeps them two blocks.
fn refuse_each(holes: &[Hole], source: &str, path: &Path) -> Outcome {
    for (written, hole) in holes.iter().enumerate() {
        if written > 0 {
            eprintln!();
        }
        refuse(&hole.diagnostic(), source, path);
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
