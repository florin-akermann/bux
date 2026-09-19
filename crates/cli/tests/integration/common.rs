//! Helpers shared by the binary's behaviour tests.
//!
//! Each test works in a directory of its own under the system's temporary directory, so no two
//! tests can see each other's files and nothing is written inside the repository.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

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

/// Runs the binary with `arguments` and returns what it said and how it exited.
pub fn lumen(arguments: &[&str]) -> Run {
    finished(running(arguments))
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
