//! The stage of the Bux command line, which `bux_command`, `bux_launcher`, and `bux_bootstrap`
//! hold to `lumen`, and in which `bux_runner` starts the runner.
//!
//! `docs/specs/run.md` states the behaviour. One JVM answers a whole list of command lines through
//! the driver `answers.lm`, and two such JVMs share a list. The Rust side answers the same list, and
//! each side works in copies of every fixture of its own.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use super::{as_argument, files_ending_in, jdk, lumen, repository};

/// The directories whose every `.lm` file is a fixture, the Bux compiler's own modules among them.
pub const FIXTURES: [&str; 3] = ["tests/spec", "library", "compiler"];

/// What the Bux command line is built from and reads as resources, and the runner under `tests/`
/// with the example program it holds, which are copied to its stage.
const CARRIED: [&str; 5] = ["compiler", "library", "bin", "tests", "example"];

/// The program that answers every command line it is handed in one JVM.
const DRIVER: &str = include_str!("../answers.lm");

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

/// The same answers from `lumen` and from the launcher of `stage` on every case.
pub fn the_same_from_the_launcher(stage: &Stage, cases: &[Case]) {
    let (rust, bux) = side_by_side(
        || {
            shard_by_shard(cases, |case, shard| {
                stage.said_by_rust(case, shard, Home::Inherited)
            })
        },
        || shard_by_shard(cases, |case, shard| stage.said_by_the_launcher(case, shard)),
    );

    the_same_answers(cases, &rust, &bux);
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

    /// The stage, with `LAUNCHED` built there by the launcher of `builder`, which is the next
    /// stage of the bootstrap that `docs/specs/run.md` states.
    pub fn built_by(builder: &Self) -> Self {
        let stage = Self::staged();
        let program = stage.root.join(LAUNCHED);
        let line = Case {
            words: vec!["build".to_owned(), as_argument(&program).to_owned()],
        };

        let built = builder.said_by_the_launcher(&line, 0);

        assert_eq!(built.status, 0, "{LAUNCHED}: {}", built.errors);
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
    let fixtures = files_ending_in("lm", &repository(), directories);
    commands
        .iter()
        .flat_map(|command| {
            fixtures
                .iter()
                .map(move |fixture| Case::written(&format!("{command} {fixture}")))
        })
        .collect()
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
