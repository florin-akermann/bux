//! Helpers shared by the binary's behaviour tests.
//!
//! Each test works in a directory of its own under the system's temporary directory, so no two
//! tests can see each other's files and nothing is written inside the repository.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

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

// The stage of the Bux command line, which `bux_command` and `bux_launcher` hold to `lumen`.
//
// `docs/specs/run.md` states the behaviour. One JVM answers a whole list of command lines through
// the driver `answers.lm`, and two such JVMs share a list. The Rust side answers the same list, and
// each side works in copies of every fixture of its own.

/// The directories whose every `.lm` file is a fixture, the Bux compiler's own modules among them.
pub const FIXTURES: [&str; 3] = ["tests/spec", "library", "compiler"];

/// What the Bux command line is built from and reads as resources, which is copied to its stage.
const CARRIED: [&str; 5] = [
    "compiler",
    "library",
    "crates/cli/src/help",
    "crates/diagnostics/src/explanations",
    "bin",
];

/// The program that answers every command line it is handed in one JVM.
const DRIVER: &str = include_str!("answers.lm");

/// A program that writes each of its arguments on a line, which the `run` command lines start.
const ECHO: &str = "import io\n\nfn main(arguments: List<String>) -> Int {\n    for argument in arguments {\n        io.println(argument)\n    }\n    0\n}\n";

/// How many shards a list of command lines is dealt into, round.
///
/// A shard is one JVM of the driver on the Bux side and one thread on the Rust side, and each
/// works in a copy of every fixture of its own. So no two command lines that write beside a
/// source, a build and a run, ever write in one directory at once.
pub const SHARDS: usize = 2;

/// What the driver's JVM is started with besides the class path.
///
/// Every test of the suite shares the machine, and a JVM spends more of it compiling and
/// collecting on threads of its own than on the command lines: one collector thread and two
/// compiler threads halve what a driver spends on `check` over every fixture, measured, and cost
/// it a tenth of its time.
const DRIVER_FLAGS: [&str; 3] = [
    "--enable-preview",
    "-XX:+UseSerialGC",
    "-XX:CICompilerCount=2",
];

/// The program the launcher starts, and the one the driver is.
pub const LAUNCHED: &str = "compiler/main.lm";
pub const DRIVEN: &str = "compiler/answers.lm";

/// What the Rust side and the Bux side each work in, each a copy of every fixture.
pub const RUST: &str = "rust";
pub const BUX: &str = "bux";

/// The same answers from `lumen` and from the driver on every case, and the stage they were
/// given in, which is nothing where no JDK can run the driver.
pub fn the_same_from_the_driver(cases: &[Case], needs: &str) -> Option<Stage> {
    let Some(java) = jdk() else {
        eprintln!("skipped: JAVA_HOME names no JDK, and {needs}");
        return None;
    };
    let stage = Stage::built(DRIVEN);

    let (rust, bux) = side_by_side(
        || {
            shard_by_shard(cases, |case, shard| {
                stage.said_by_rust(case, shard, Home::Inherited)
            })
        },
        || stage.said_by_the_driver(&java, cases, Home::Inherited),
    );

    the_same_answers(cases, &rust, &bux);
    Some(stage)
}

/// What the Rust side and the Bux side say, each worked out on a thread of its own at once.
pub fn side_by_side(
    rust: impl FnOnce() -> Vec<Said> + Send,
    bux: impl FnOnce() -> Vec<Said> + Send,
) -> (Vec<Said>, Vec<Said>) {
    thread::scope(|scope| {
        let rust = scope.spawn(rust);
        let bux = bux();
        (rust.join().expect("the Rust side answers"), bux)
    })
}

/// What `answer` gives for each case, each shard on a thread of its own, every case of one shard
/// in order in that shard's copy.
pub fn shard_by_shard(cases: &[Case], answer: impl Fn(&Case, usize) -> Said + Sync) -> Vec<Said> {
    let mut answers: Vec<Option<Said>> = cases.iter().map(|_| None).collect();
    thread::scope(|scope| {
        let answer = &answer;
        let shards: Vec<_> = (0..SHARDS)
            .map(|shard| {
                scope.spawn(move || {
                    dealt(cases.len(), shard)
                        .map(|index| (index, answer(&cases[index], shard)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for shard in shards {
            for (index, said) in shard.join().expect("a shard answers") {
                answers[index] = Some(said);
            }
        }
    });
    answers
        .into_iter()
        .map(|said| said.expect("every case is answered"))
        .collect()
}

pub fn the_same_answers(cases: &[Case], rust: &[Said], bux: &[Said]) {
    let differ: Vec<String> = cases
        .iter()
        .zip(rust.iter().zip(bux))
        .filter(|(_, (rust, bux))| rust != bux)
        .map(|(case, (rust, bux))| {
            format!("lumen {}\n rust: {rust:?}\n bux:  {bux:?}", case.shown())
        })
        .collect();
    assert!(
        differ.is_empty(),
        "{} of {} command lines differ:\n{}",
        differ.len(),
        cases.len(),
        differ[..differ.len().min(5)].join("\n")
    );
}

/// The words of one command line.
#[derive(Clone)]
pub struct Case {
    words: Vec<String>,
}

impl Case {
    pub fn written(line: &str) -> Self {
        Self {
            words: line.split_whitespace().map(str::to_owned).collect(),
        }
    }

    fn shown(&self) -> String {
        self.words.join(" ")
    }
}

/// A directory of its own that holds the Bux command line, built, and a copy of every fixture
/// for each shard of each side to work in, so no command line sees what another one writes.
pub struct Stage {
    pub root: PathBuf,
}

impl Stage {
    /// The stage, with `program` built there: `LAUNCHED` for the launcher, or `DRIVEN`.
    pub fn built(program: &str) -> Self {
        let stage = Self::staged();
        let built = lumen(&["build", as_argument(&stage.root.join(program))]);
        assert_eq!(built.code, 0, "{program}: {}", built.stderr);
        stage
    }

    /// The stage, with nothing built there yet.
    pub fn staged() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lumen-bux-command-{}-{}",
            std::process::id(),
            STAGES.fetch_add(1, Ordering::Relaxed)
        ));
        for directory in CARRIED {
            copied(&repository().join(directory), &root.join(directory));
        }
        fs::write(root.join(DRIVEN), DRIVER).expect("the driver is writable");
        let stage = Self { root };
        for side in [RUST, BUX] {
            for shard in 0..SHARDS {
                let within = stage.side(side, shard);
                for directory in FIXTURES {
                    copied(&repository().join(directory), &within.join(directory));
                }
                fs::write(within.join("echo.lm"), ECHO).expect("a program is writable");
            }
        }
        stage
    }

    /// What `lumen` says to `case`, started in the Rust side's copy for `shard`.
    pub fn said_by_rust(&self, case: &Case, shard: usize, home: Home<'_>) -> Said {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
        command
            .args(&case.words)
            .current_dir(self.side(RUST, shard));
        Said::of(&finding(command, home))
    }

    /// What `bin/bux` says to `case`, started in the Bux side's copy for `shard`.
    pub fn said_by_the_launcher(&self, case: &Case, shard: usize) -> Said {
        self.said_by_the_launcher_finding(case, shard, Home::Inherited)
    }

    /// The same, with `JAVA_HOME` holding what `home` says.
    pub fn said_by_the_launcher_finding(&self, case: &Case, shard: usize, home: Home<'_>) -> Said {
        let mut command = Command::new(self.root.join("bin/bux"));
        command.args(&case.words).current_dir(self.side(BUX, shard));
        Said::of(&finding(command, home))
    }

    /// What the Bux command line says to each case, from one JVM for each shard.
    pub fn said_by_the_driver(&self, java: &Path, cases: &[Case], home: Home<'_>) -> Vec<Said> {
        let answers = self.root.join("answers");
        fs::create_dir_all(&answers).expect("the answers' directory is creatable");
        let paths: Vec<PathBuf> = cases
            .iter()
            .enumerate()
            .map(|(index, case)| {
                let path = answers.join(index.to_string());
                fs::write(&path, case.words.join("\n")).expect("a case is writable");
                path
            })
            .collect();
        thread::scope(|scope| {
            for shard in 0..SHARDS {
                let dealt: Vec<&PathBuf> = dealt(paths.len(), shard)
                    .map(|index| &paths[index])
                    .collect();
                scope.spawn(move || self.drive(java, &dealt, Driving { shard, home }));
            }
        });
        paths.iter().map(|path| answer_beside(path)).collect()
    }

    /// One JVM of the driver, handed the cases at `paths`, started in the copy of its shard.
    fn drive(&self, java: &Path, paths: &[&PathBuf], driving: Driving<'_>) {
        if paths.is_empty() {
            return;
        }
        let class_path = std::env::join_paths([self.root.join("compiler"), self.root.clone()])
            .expect("a class path joins");
        let mut command = Command::new(java);
        command
            .args(DRIVER_FLAGS)
            .arg("-cp")
            .arg(class_path)
            .arg("answers")
            .args(paths)
            .current_dir(self.side(BUX, driving.shard));
        let ran = finding(command, driving.home);
        assert!(
            ran.status.success(),
            "{}",
            String::from_utf8_lossy(&ran.stderr)
        );
    }

    /// Where one shard of one side works: its own copy of every fixture.
    pub fn side(&self, side: &str, shard: usize) -> PathBuf {
        self.root.join(side).join(shard.to_string())
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

static STAGES: AtomicUsize = AtomicUsize::new(0);

/// The shard one driver works in, and what `JAVA_HOME` holds for it.
#[derive(Clone, Copy)]
struct Driving<'h> {
    shard: usize,
    home: Home<'h>,
}

/// What `JAVA_HOME` holds for a command: what it holds for the tests, nothing, or a path.
#[derive(Clone, Copy, Debug)]
pub enum Home<'h> {
    Inherited,
    Unset,
    At(&'h str),
}

/// The command's output, with no input, and with `JAVA_HOME` holding what `home` says.
fn finding(mut command: Command, home: Home<'_>) -> Output {
    command.stdin(Stdio::null());
    match home {
        Home::Inherited => {}
        Home::Unset => {
            command.env_remove("JAVA_HOME");
        }
        Home::At(path) => {
            command.env("JAVA_HOME", path);
        }
    }
    command.output().expect("the command starts")
}

/// What one command line wrote on each stream, and the status it ended with.
///
/// A stream that is not UTF-8 is kept as its bytes, so two streams are equal only byte for byte.
#[derive(Debug, PartialEq, Eq)]
pub struct Said {
    pub output: String,
    pub errors: String,
    pub status: i32,
}

impl Said {
    fn of(output: &Output) -> Self {
        Self {
            output: text_of(output.stdout.clone()),
            errors: text_of(output.stderr.clone()),
            status: output.status.code().unwrap_or(-1),
        }
    }
}

/// What the driver wrote beside the case at `path`.
fn answer_beside(path: &Path) -> Said {
    let read = |extension: &str| {
        fs::read(path.with_extension(extension)).expect("the driver answers every case")
    };
    Said {
        output: text_of(read("out")),
        errors: text_of(read("err")),
        status: text_of(read("status"))
            .parse()
            .expect("a status is a number"),
    }
}

fn text_of(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap_or_else(|error| format!("{:?}", error.into_bytes()))
}

/// The indices of the cases dealt to `shard`, in order, out of `count` dealt round.
fn dealt(count: usize, shard: usize) -> impl Iterator<Item = usize> {
    (shard..count).step_by(SHARDS)
}

/// Each command of `commands` on every `.lm` file under `directories`, by its path from the
/// root of a side.
pub fn fixture_cases(commands: &[&str], directories: &[&str]) -> Vec<Case> {
    let mut fixtures = Vec::new();
    for directory in directories {
        gather(&repository(), Path::new(directory), &mut fixtures);
    }
    fixtures.sort();
    commands
        .iter()
        .flat_map(|command| {
            fixtures
                .iter()
                .map(move |fixture| Case::written(&format!("{command} {fixture}")))
        })
        .collect()
}

/// Every `.lm` file under `directory`, however deep, by its path from `root`.
fn gather(root: &Path, directory: &Path, found: &mut Vec<String>) {
    for entry in fs::read_dir(root.join(directory)).expect("a fixture directory is readable") {
        let name = entry.expect("a directory entry is readable").file_name();
        let path = directory.join(&name);
        if root.join(&path).is_dir() {
            gather(root, &path, found);
        } else if path.extension().is_some_and(|extension| extension == "lm") {
            found.push(path.display().to_string());
        }
    }
}

/// A copy of the directory `from` at `to`, leaving out class files and hidden entries.
fn copied(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("a directory of the stage is creatable");
    for entry in fs::read_dir(from).expect("a directory of the repository is readable") {
        let path = entry.expect("a directory entry is readable").path();
        let name = path.file_name().expect("an entry has a name");
        let hidden = name.to_string_lossy().starts_with('.');
        if hidden
            || path
                .extension()
                .is_some_and(|extension| extension == "class")
        {
            continue;
        }
        if path.is_dir() {
            copied(&path, &to.join(name));
        } else {
            fs::copy(&path, to.join(name)).expect("a file of the repository is copyable");
        }
    }
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
