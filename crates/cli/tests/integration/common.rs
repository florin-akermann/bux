//! Helpers shared by the binary's behaviour tests.
//!
//! Each test works in a directory of its own under the system's temporary directory, so no two
//! tests can see each other's files and nothing is written inside the repository.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A module written beside an example, which is what an import of that name finds.
pub struct Sibling<'w> {
    pub named: &'w str,
    pub content: &'w str,
}

/// A file written under an example's directory, which is what a manifest and a package are.
pub struct Within<'w> {
    /// Where the file goes, read against the example's own directory.
    pub at: &'w str,
    pub content: &'w str,
}

/// A source file of `content`, in a directory no other test writes to.
pub struct Example {
    pub directory: PathBuf,
    pub path: PathBuf,
}

impl Example {
    pub fn new(content: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "lumen-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&directory).expect("a temporary directory is creatable");
        let path = directory.join("example.lm");
        std::fs::write(&path, content).expect("an example is writable");
        Self { directory, path }
    }

    /// The same, with `sibling` written into the directory the example is in.
    pub fn beside_it(&self, sibling: &Sibling<'_>) {
        let path = self.directory.join(sibling.named).with_extension("lm");
        std::fs::write(path, sibling.content).expect("a module beside an example is writable");
    }

    /// The same, with `file` written where it says under the example's directory.
    ///
    /// A package's manifest and a package it depends on are each written this way, because
    /// neither is a module and a dependency is a directory of its own.
    pub fn within_it(&self, file: &Within<'_>) {
        let path = self.directory.join(file.at);
        if let Some(holding) = path.parent() {
            std::fs::create_dir_all(holding).expect("a directory under an example is creatable");
        }
        std::fs::write(path, file.content).expect("a file under an example is writable");
    }

    /// What the file holds now.
    pub fn content(&self) -> String {
        std::fs::read_to_string(&self.path).expect("an example is readable")
    }

    /// What was written beside the example at `path`, when anything was.
    pub fn beside(&self, path: &str) -> Option<Vec<u8>> {
        std::fs::read(self.directory.join(path)).ok()
    }
}

impl Drop for Example {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// What `program` writes for each of `sources`, in order, from one run of the binary.
///
/// Each source goes to a file of its own under the program's directory, and the one run is handed
/// every file, because each run is one compiler and one JVM. The program writes its answer for a
/// file at the file's path with `.answer` added.
pub fn answers_of_one_run(program: &Example, sources: &[String]) -> Vec<String> {
    let inputs: Vec<PathBuf> = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let at = format!("inputs/{index}.txt");
            program.within_it(&Within {
                at: &at,
                content: source,
            });
            program.directory.join(at)
        })
        .collect();
    let mut arguments = vec!["run", as_argument(&program.path)];
    arguments.extend(inputs.iter().map(|input| as_argument(input)));

    let run = lumen(&arguments);

    assert_eq!(run.code, 0, "{}", run.stderr);
    inputs
        .iter()
        .map(|input| {
            std::fs::read_to_string(input.with_extension("txt.answer"))
                .expect("the program writes a UTF-8 answer for each input")
        })
        .collect()
}

/// Runs the binary with `arguments` and returns what it said and how it exited.
pub fn lumen(arguments: &[&str]) -> Run {
    finished(running(arguments))
}

/// The same run, started in `directory`, which is where a program writes what it writes.
///
/// A program reaches the file system through a path of its own, and a relative one is read
/// against the directory the program was started in. Starting it in its own directory is what
/// keeps a run that writes a file from writing it inside the repository.
pub fn lumen_within(arguments: &[&str], directory: &Path) -> Run {
    let mut command = running(arguments);
    command.current_dir(directory);
    finished(command)
}

/// The same run, as if `JAVA_HOME` named `home`, or named nothing at all.
pub fn lumen_finding(arguments: &[&str], home: Option<&Path>) -> Run {
    let mut command = running(arguments);
    match home {
        Some(home) => command.env("JAVA_HOME", home),
        None => command.env_remove("JAVA_HOME"),
    };
    finished(command)
}

/// The `java` of the JDK `JAVA_HOME` names, which a test needs to run anything it built.
pub fn jdk() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var_os("JAVA_HOME")?);
    let java = home.join("bin").join("java");
    java.is_file().then_some(java)
}

/// The root of the repository, where `compiler/` and `tests/spec/` are.
pub fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A path as an argument of the binary.
pub fn as_argument(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 path")
}

fn running(arguments: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
    command.args(arguments);
    command
}

fn finished(mut command: Command) -> Run {
    Run::of(&command.output().expect("the lumen binary runs"))
}

/// What one run of the binary amounted to.
pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    fn of(output: &Output) -> Self {
        Self {
            code: output.status.code().expect("the binary exits with a code"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}
