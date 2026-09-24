# Bux: Implementation and Roadmap

The language is specified in `docs/design.md`; this document covers how it is built and shipped.

## 1. JVM target

The JVM is the initial and primary runtime target, and Valhalla is what Bux is built on.

**JDK 28 or later is targeted**, early access until it ships.
The emitted class-file version is 72, and no older JVM is supported or tested.
Every class the compiler writes is a value class, which JDK 28 holds in preview.
That preview is JEP 401, Value Classes and Objects, with JEP 539, Strict Field Initialization.
Every field the compiler writes is strict, which JEP 401 asks of every value class.
A preview class file carries minor version 65535, and a JVM loads one only when told to.
`bux run` starts the JVM with `--enable-preview`, so a run needs nothing the reader must know.
Modern JVM features are used freely: invokedynamic, records, sealed classes, value classes.
A release of the JDK moves the target with it; there is no compatibility matrix and never will be.
The `jar` line of a manifest, which `docs/specs/packages.md` states, is for the JVM target alone.

Initial pipeline:

```text
Source
  ↓
Lexer
  ↓
Parser
  ↓
AST
  ↓
Canonical-form check
  ↓
Name Resolution
  ↓
HIR
  ↓
Type Inference
  ↓
Typed HIR
  ↓
Exhaustiveness Checking
  ↓
Lowering
  ↓
JVM IR
  ↓
JVM Bytecode
```

---

## 2. Memory management

There is no manual memory management.

The JVM owns memory management.

Programmers should not think about:

* ownership
* borrowing
* lifetimes
* explicit allocation/deallocation
* garbage collectors

The compiler may optimize representations where possible.

For example, `type UserId = UserId(Int)` has compile-time semantics distinct from `Int`.
The compiler may still represent it as a JVM primitive or value type.

---

## 3. Java interoperability

Java interoperability is important because the JVM ecosystem is valuable.

However, Java APIs should have a clear boundary.
`docs/specs/interop.md` states the `extern` declaration, which names one member of one Java class.

The standard library should provide idiomatic wrappers around common Java APIs.
Users should not be forced to interact directly with Java's object model.
A program calls `http.get(url)`, and the library holds the `HttpClient` builder under it.
A program reaches the platform through the library, so only `library/` declares an `extern`.
The compiler is the first program held to it, and `tests/conventions.bx` refuses any other file.

Java interop should be powerful but should not determine the design of the language.


## 4. Standard library

The standard library is small, and its data structures are smaller still.
The fewer functions a module offers, the better; a `for` loop does the rest.
A function lands only where a plain loop over what the type already exposes cannot write it.
A map has no iterator, and the same holds for every convenience a loop already writes.
No function takes another function, so the library has no `map` and no `filter`.
`docs/principles.md` question 12 is what a proposed function answers.

A data structure the library holds has the best asymptotic cost known for what it does.
A map looks a key up in constant time, and a list is read at an index in constant time.
The implementation is a textbook one, written as a school project writes it: plain and correct.
Tuning beyond that cost is refused until a measurement on a real program asks for it.

The library stands on files, programs, streams, the environment, the clock, and text.
It also stands on Java archives and on the bits of a whole number, which the compiler needs.
`docs/specs/io.md` names the module and the `extern` declarations of each of these.
Before Item 128, `library/` declared 78 `extern` declarations, `compiler/` 43, and `tests/` 21.
After it, `library/` declares 117, and no file outside it and `tests/spec/` declares one.
The 25 that went were second declarations of a class or a member that the library held already.

The initial areas are strings, collections, `Option`, `Result`, IO, and files.
HTTP, time, JSON, and concurrency are initial areas too.

The language should make it easy to consume JVM libraries.
It should not attempt to recreate the entire Java ecosystem.

---

## 5. Tooling

There should be one primary executable:

```text
bux
```

Commands:

```text
bux build
bux run
bux test
bux fmt
bux check
bux api
bux repl
bux add
```

The project should avoid requiring users to understand:

```text
javac
jar
maven
gradle
kotlinc
```

for normal language development.

---

## 6. Compiler implementation

The compiler is written in Bux, and it compiles itself.
Self-hosting came before every feature the self-hosted compiler did not need.
Section 11 is the ladder of items it needed, and Item 075 completed it.

The compiler runs on the JVM under `--enable-preview`, started by the `bux` launcher.
No native binary is needed to compile the compiler with itself.
GraalVM native-image waits until GraalVM tracks JDK 28 and Valhalla, and section 12 holds it.

The compiler lives in `compiler/`, one module for each phase, and the lexer is the first.
`compiler/main.bx` is its command line, and `compiler/command.bx` holds every command.
A build writes every class under `target/`, in the directory of the module it builds.
So a build of the compiler writes `compiler/target/`, and no class lands beside a source.
`compiler/help/` and `compiler/explanations/` hold the help text and the long form of each code.
`library/` at the root holds the library, and the compiler reads all three as resources.
A module there has a `.bx` name, and the module loader reads no other extension.
A build copies the three from its own class path into `target/`, beside the classes it writes.
The launcher `bin/bux` starts the compiler with `compiler/target/` alone on the class path.
Run `bin/bootstrap` after a change to a resource, because only it copies those of the checkout.

What remains of the repository is small, and a JDK is the one tool it needs:

```text
bin/        bootstrap, bux, and the seed seed.jar
compiler/   the compiler, its help text, and its explanations
library/    the standard library, read by the compiler as a resource
tests/      the tests of the compiler, and tests/spec, the executable examples
example/    the example program
docs/       the specification, this document, and the behaviour specs
```

### The bootstrap

The bootstrap has a seed and two stages, and `bin/bootstrap` runs them.
The seed `bin/seed.jar` holds the classes of a compiler that an earlier compiler built.
The seed runs from one temporary directory: the seed unpacked, with the resources of the checkout.
The seed builds stage 1, the classes in `compiler/target/`.
Stage 1 builds stage 2 from a copy of `compiler/` outside the repository.
Stage 2 must be stage 1 byte for byte, and `docs/specs/run.md` states the script.
No class file other than the seed is kept in the repository, so a checkout bootstraps first.

The first seed is the compiler that the Rust compiler built, before Item 087 deleted the Rust.
A change to the compiler changes stage 1, and stage 2 still equals it, because stage 1 built it.
So the seed is replaced for one of six reasons, and for no other.
The first reason is that the compiler needs a feature that the seed cannot compile.
The item that needs it builds stage 1 with the old seed, and it packs stage 1 as the new seed.
The second reason is that the lowering or the writer changed the bytes of the compiler's classes.
Then stage 1, which the old seed wrote, differs from stage 2, which the changed compiler wrote.
The item builds stage 2 with stage 1, and stage 3 with stage 2, and stage 3 must equal stage 2.
It packs stage 2 as the new seed, with the `jar` tool of the JDK.
Item 093 did this when `list.length` became a read of the length field.
The old seed calls the `length` that the old `library/list.bx` declared.
So the old seed built stage 1 beside the old `library/list.bx`, not beside the new one.
The third reason is that a build writes its classes to another place.
The old seed writes stage 1 where the new `bin/bootstrap` does not read it.
The item builds stage 1 with the old seed, stage 2 with stage 1, and stage 3 with stage 2.
Stage 3 must equal stage 2, and the item packs stage 2 as the new seed.
Item 096 did this when a build moved every class from beside its source to `target/`.
Item 113 did this when a build first copied the resources into `target/`.
The fourth reason is that canonical form changed, so the old seed refuses the sources.
The item changes the printer, and the old seed builds stage 1 from the sources in the old form.
The `bux fmt` of stage 1 then writes every source in the new form.
Stage 1 builds stage 2, stage 2 builds stage 3, and stage 3 must equal stage 2.
The item packs stage 2 as the new seed.
Item 092 did this when the blank line between two imports was taken out.
The fifth reason is that the sources or the classes get new names.
The old seed reads no source of the new name, or it writes each class under its old name.
The item changes the compiler first, and the old seed builds stage 1 from the sources as named.
Then the sources get their new names, stage 1 builds stage 2, and stage 2 builds stage 3.
Stage 3 must equal stage 2, and the item packs stage 2 as the new seed.
Item 081 did this when every source got the extension `.bx` and the JVM package became `bux/`.
The sixth reason is that the syntax changed, so the old seed cannot read the sources in the new.
The item first changes the compiler to read the old syntax and the new, and to print the new.
The old seed builds stage 1, and the `bux fmt` of stage 1 writes every source in the new syntax.
Stage 1 builds stage 2, and the item then takes the old syntax out of the compiler.
Stage 2 builds stage 3, stage 3 builds stage 4, and stage 4 must equal stage 3.
The item packs stage 3 as the new seed.
Item 082 did this when a name that never changes was first bound with `let`.
In each case, `bin/bootstrap` must then hold stage 2 equal to stage 1 with the new seed.
The item says so, and it replaces the seed in its own commit.
The seed packs `compiler/target/` whole, and `bin/bootstrap` replaces its copies of the resources.

The compiler should itself use strong typed representations for compiler phases.

Avoid passing loosely typed structures between phases.

Prefer:

```text
UntypedAst
    ↓
ResolvedAst
    ↓
TypedAst
    ↓
LoweredIr
    ↓
ClassFile
```

rather than one mutable AST that means different things at different stages.

The last two phases are the lowering, `compiler/ir.bx`, and the writer, `compiler/jvm.bx`.
The lowering names a local by its name, a call by its function, and a type by its Bux type.
So its tree has no field for a JVM class, a descriptor, a slot, or a `java/` string.
The writer names each of them: it gives each local its slot and each call its descriptor.
It holds every JVM spelling, such as the class of a mailbox and the `<init>` of a constructor.
Then it assembles the instructions and writes the class file.
So a change to how a class file spells a thing is a change to the writer alone.
Two Bux values can have one descriptor, so the writer keeps one method of each name and descriptor.

---

## 7. Testing strategy

Compiler development is specification-driven, and the executable examples are the specification.
`docs/specs/executable-examples.md` says what an example file is and what it expects.
An example is a `.bx` file under `tests/spec/<area>/`, and an area exists when an example needs it.

### The tests

The compiler's tests are the tests of `tests/`, and `bin/bux test tests` runs them all.
`tests/` is a package, and `docs/specs/testing.md` states a test and a run of a package.
The run starts from the root of the repository, so each path of a test reads from there.
Each module of `tests/` runs in a JVM of its own, which calls the functions of the compiler.
So a test starts no compiler process for each file that it checks.
Each check is a `test` block of the module that holds it:

- `exemplified.bx`: every example under `tests/spec`, to its header and to canonical form.
- `siblings.bx`: every sibling file, to the view of its phase: tokens, tree, surface, format.
- `documented.bx`, `compiler_lines.bx`, and `spec_lines.bx`: every `// example:` line.
  They hold `library/`, `compiler/`, and `tests/spec`, one place each.
  They also run every `test` block of those modules, except the tests under `tests/spec/`.
- One module for each phase, with a test for each drawn property of that phase.
- `commanded.bx` and `fixtures.bx`: the command line, held to the golden answers.
  The golden answers are under `tests/commands/`.
- `enacted.bx`, `started.bx`, and `archived.bx`: each command, done in a directory of its own.
- `launched.bx`: the launcher `bin/bux`, and the time limit that a started program has.

A check gives back a `Held`: the cases it tried, a line for each failure, and each skip.
`held.is_held_reported` writes a line for each skip and each failure of a check.
The test holds where no case failed.
`bux test` passes on the lines that a test writes, so a failure names its place and its cause.
A skip is a line `skipped: <why>`, and the test holds, as `library/prelude.bx` shows.
`bux test` runs the modules on a pool of one worker for each processor, as Item 112 made it.
The long checks are in modules of their own, so the pool runs them beside each other.
The order of the report is the order of the modules, so no line depends on which worker ends first.

`documented.bx` holds one memo of typed modules, `command.Memo`, for each test, from call to call.
The memo holds each module by the canonical path of its file, so each module is typed once a test.
It also holds each module that `whole` found whole, and `whole` is not run on it again.
A command starts with an empty memo, so no answer of the command line depends on one.
`documented.bx` writes the run of each module, as `bux test` writes it, into a directory.
Each run gets a directory of its own, inside one directory made for the runs of its test.
Then the test starts each run in a JVM of its own, with the line that `bux test` uses.
So the class path of a run is its own directory, with the resources, and no two runs share a class.
One build of a package that all its runs share would not be correct, so each run writes its own.
A run asks generics for types that no build of the package asks for, such as in an example.
The lowering writes each such generic into the class of the module that declares it.
The JVM starts in the root of the repository, so a run finds a file where `bux test` finds it.
The test reads what the JVM wrote as `bux test` reads it, so a failure names its module.
A run that ends with a status other than 0, such as after a stack overflow, is a failure.
A run past the limit is a failure too, and in both cases the next run starts.
A module whose run cannot be built is reported as `bux test` reports it, and the others run.
The tests use no reflection, no class loader, and no `java.net`, because Item 084 refuses them.

`walk.completed_from` starts each program that a test starts, and gives each a time limit.
Each program starts with the environment that `command.cleared` leaves, as a run of `bux test` does.
`walk.start_limit` is the one limit, in seconds, and every started program gets it.
A program that is still running at the limit is stopped, and it is a failure that names its check.
The test then continues with the next check, so one program that never ends cannot stop it.
The limit is 60 s, because it guards against a hang, and not against a slow program.
The time of a program grows with machine load, and two suites often run at the same time.
So a limit near the time of the longest program would stop a correct run.
Before Item 094, the longest program took 0.14 s, and the limit was 1 s.
`launched.bx` holds the mechanism: a program that never ends is stopped at a limit of 1 s.
The check holds that the report comes after 1 s at least, and before the 60 s of `start_limit`.
Until Item 103 the window was 2 s, and it failed under a full pool.
A worker process that waits gets a processor back only when another worker lets one go.
On 2026-09-24 the report came after 1.1 s in six runners at once, with a load of 62 on 12 cores.
With 12 processes that never let a processor go beside it, the report came after 10.2 s.
So the window is 60 s wide, and a limit counted in minutes, not seconds, still fails it.
`bin/bux run tests/golden.bx` writes each golden file again, for an answer that changes on purpose.
It also adds each command line on an example that `fixtures.txt` does not hold.

### The runner, until Item 114

Until Item 114, `tests/runner.bx` held these checks, and `bin/runner` built it and started it.
It split the checks into jobs and ran them on a pool of its own, one worker for each processor.
The example lines and the examples were split into as many jobs as there were workers.
Item 114 made each check a test, so `bux test` is the one runner, as every Bux program has.
On 2026-09-24 `bin/runner` took 262 s, 211 s, and 147 s, and the median was 211 s.
After Item 114, `bin/bux test tests` took 176 s, 118 s, and 118 s, and the median was 118 s.
The runs of before and after took turns, under a load of 40 to 65 on 12 cores from other work.
One module held every example line of `tests/spec`, `library/`, and `compiler/` at first.
It took 91 s alone, so `compiler_lines.bx` and `spec_lines.bx` now hold two of those places.
`fixtures.bx` holds the longest golden file, so it runs beside `commanded.bx` too.
On 2026-09-23 `bin/runner` took 182 s before Item 093 and 97 s after it.
On 2026-09-23 `bin/runner` took 98 s before Item 094 and 53 s after it.
The example lines took 78 s before Item 094 and 34 s after, and no other part changed by over 1 s.
On 2026-09-24 `bin/runner` took 64 s before Item 091 and 31 s after it.
On 2026-09-24 the median of three `bin/runner` wall times was 56 s before Item 103 and 43 s after.
On 2026-09-24 the median of three `bin/runner` wall times was 37 s before Item 084.
After Item 084, with one JVM for each run, the median of three wall times was 40 s.
On 2026-09-24 `bin/bootstrap` took 5.6 s before Item 091 and 5.8 s after it.
Item 091 did not put the compiler on processes, because a measurement showed no gain.
A build of the compiler wrote its 1400 classes in 0.31 s on one thread and on 1400 processes.
The processes spent four times the processor time, most likely in code the JIT had not compiled.
Before Item 110 `bin/bootstrap` compared the stages with one `cmp` for each class.
Item 110 compares them with one `diff -rq` over the two `target/` directories.
On 2026-09-24 it compared 923 classes, and one `diff -rq` took 0.08 s.
The median of three `bin/bootstrap` wall times was 7.6 s before Item 110 and 6.2 s after.
The runs of before and after took turns, under a load of 11 to 12 on 12 cores from other work.
On 2026-09-24 `bux build compiler/main.bx` took 18.8 s before Item 126 and 18.9 s after it.
Each is the median wall time of three runs that took turns, under a load of 54 to 68.
The medians of processor time were 15.2 s and 17.5 s, but after built its own tree, which is larger.
On the sources of before, the two compilers used 16.8 s and 16.9 s, the medians of three runs.
So the writer that gives each slot and descriptor costs no time that this load can show.

### The profile of 2026-09-24

Item 111 profiled an Apple M4 Pro with 12 processors, on JDK 28-ea+16, under a load of 10 to 30.
Each number is a median of two runs or more of `bin/bux` or `bin/runner`, with this flag added.
`-XX:StartFlightRecording=filename=build.jfr,settings=profile,jdk.ExecutionSample#period=1ms`.
`jfr print --json --stack-depth 4096 --events jdk.ExecutionSample build.jfr` gives the samples.
Loading is `lexer`, `parser`, `format`, `ast`, `modules`, and the rest of `command`.
Resolving is `resolver`, lowering is `ir`, and class writing is `jvm`, `bytes`, and disk writes.
Typing is `declared`, `infer`, `unify`, `types`, `surface`, `exhaustiveness`, and `holes`.
It is also `escapes`, `carried`, `boundary`, `refusal`, and `processes`.
The job held the 25 modules of `compiler/`, and it staged 24 runs with the class path of the runner.

| Phase | Build of `compiler/main.bx` | Job over `compiler/` | Runner pass | `bux test tests` |
|-------|-----------------------------|----------------------|-------------|------------------|
| Typing | 37%, 1.28 s | 22%, 2.4 s | 22% | 44% |
| Loading | 26%, 0.89 s | 31%, 3.4 s | 28% | 30% |
| Class writing | 16%, 0.54 s | 21%, 2.4 s | 20% | 7% |
| Of it, disk writes | 2% to 4% | 2%, 0.2 s | 4% | 1% |
| Lowering | 13%, 0.46 s | 22%, 2.5 s | 24% | 10% |
| Resolving | 9%, 0.32 s | 4%, 0.5 s | 5% | 10% |
| JVM starts and runs | none | 24 runs, 3.3 s | 401 JVMs | 43 JVMs, 9.9 s |

The seconds of the build are its shares of 3.5 s, the wall time of its least loaded run.
With JFR the build took 3.5 s, 6.5 s, and 7.4 s, and six builds without it had a median of 5.6 s.
The seconds of the job are its shares of the 11.2 s it staged, before its 3.3 s of runs.
With JFR the job took 15.4 s, and without it 19.3 s and 16.0 s; the runner took 79.4 s and 77.9 s.
The lexer reads each source three times: `parser.over`, `format.printer_of`, `command.commented_of`.
`escapes`, reached once for each expression that inference settled, is 11% to 12% of a build.
A build used 12 s to 17 s of processor time, and 5.9 s, not 14.8 s, with `-XX:TieredStopAtLevel=1`.
So the C2 JIT uses about 9 s of each build, and a pool of workers shares the processors with it.
The collector paused a build for 0.1 s and a runner pass for 3.8 s, and used 0.6 s and 27 s.
A runner pass used 368 s of processor time, of which 275 s were in the JVM of the runner.
`jdk.ProcessStart` counts 393 `java`, 6 `jar`, and 2 `bin/bux` JVMs in a pass; 30 ran `started.bx`.
Of the other `java` ones, 163 ran example lines, 102 command lines, 90 `expect-run`, and 8 the rest.
A JVM that runs a trivial `main` with the line of `bux test` took 0.064 s, the median of 40 starts.
It used 0.08 s of processor time, so the 401 starts of a pass cost about 32 s, or 9% of it.
A build writes 575 specialized bodies of 42 generics, the methods `name$Type` of its classes.
In one pass, the 533 later copies take 17% of the code bytes, and so 0.17 s of lowering and writing.
`bux test tests` took 118 s and 110 s; with JFR it took 177 s and waited 9.9 s on its 43 JVMs.
At 0.064 s their starts cost 2.8 s; Item 123 said 0.12 s, and a start took 0.11 s at a load of 20.
Item 112 put `bux test <package>` on a pool; the medians of three, before and after, took turns.
`bux test compiler` took 28.8 s and 10.2 s, and `bux test tests` took 106.6 s and 20.8 s.
`bin/runner` took 55.7 s and 57.0 s, at a load of 4 to 22, and the pool property adds 20 s of work.

### The One Billion Row Challenge

Item 130 ran `1brc/src/main.bx` once over one billion lines; `docs/specs/billion-rows.md` states it.
A script that is not in the repository wrote the input once: 13.5 GB with 412 stations, in 625 s.
`time bin/bux run 1brc/src/main.bx <path>` took 187.3 s of wall time, and 804 s of processor time.
That is the compile of the program and its run, on 12 workers, with no flag added to the JVM.
The machine was an Apple M4 Pro with 12 processors and 24 GB, on JDK 28-ea+16.
Other work ran beside it: the load was 10 when the run started and 50 when it ended.
The line of output matched the one a Python reference script wrote from the same file.
Item 131 profiles the run, and the program is not tuned before it.
Item 132 wrote the input in Bux: `time bin/bux run 1brc/src/create_measurements.bx 1000000000`.
It wrote 13.8 GB in 288.6 s of wall time and 216 s of processor time, on one thread.
The load was 72 when the run started and 43 when it ended, on the same machine and JDK.

### The share of the machine

Item 136 bounds the heap of each JVM, because the JDK gives each one a quarter of the memory.
On 2026-09-24 two runners at once held 8 GB, and each worktree agent starts runners of its own.
The numbers are peaks of `ps -o rss`, on 12 processors and 24 GB, under a load of 30 to 140.

| JVM | Heap before | RSS before | Heap after | RSS after |
|-----|-------------|------------|------------|-----------|
| The runner | 6 GB | 6.7 GB | 4 GB | 4.6 GB |
| `bux test tests` | 6 GB | 6.7 GB | 4 GB | 4.6 GB |
| One compile of `bin/bootstrap` | 6 GB | 1.5 GB | 1 GB | 0.7 GB to 0.9 GB |
| The largest JVM the runner starts | 6 GB | 0.9 GB | 256 MB for a run | 0.1 GB |

With no bound, the runner held 5.5 GB after a collection, and 3.5 GB after a remark.
It ran out of heap at 2 GB and passed at 3 GB, and `bux test tests` ran out at 1 GB.
So `bin/runner` and `bin/bux` get 4 GB, and each compile of `bin/bootstrap` gets 1 GB.
Item 114 then deleted `bin/runner`, so `bin/bux test tests` holds the checks it held.
Each module of `tests/` runs in a JVM of its own, and that JVM has no bound yet.
`bin/runner` took 183 s at a load of 115 and 110 s after, at a load of 41; the load makes both vague.
A short run gets `-Xmx256m`, `-XX:+UseSerialGC`, and `-XX:TieredStopAtLevel=1`.
`-Xshare:auto` is refused, because the JVM turns class-data sharing off beside `--limit-modules`.

### Drawn properties

A property is a Bux function that tries one invariant on drawn input.
`tests/drawn.bx` is the generator, a linear congruential generator seeded by the case number.
A test tries its property on `held.drawn_cases` cases, 100, so two runs draw the same input.
A failure names the property, the seed, and the drawn input, so a reader can try that case again.
Round trips come first: print then parse, a format that is idempotent, spans that cover the input.

### What the Rust tests held that the tests of `tests/` do not

Item 087 moved every Rust test to the runner, which Item 114 made the tests of `tests/`.

- Shrinking: a failure shows the whole drawn input, not the smallest input that fails.
- A property that compared a Bux phase with the Rust phase, because no second phase is left.
- That comparison covered the lexer, parser, format, loader, resolver, types, checks, and classes.
- Each of their generators now drives a Bux property that holds an invariant of its own.
- The files that `build` and `fmt` write, compared byte for byte with the files Rust wrote.
- The span of a refusal, which an example does not state, except in a `.error` sibling.
- The shape of a Rust-internal value: the IR, a descriptor, and a resolved kind of definition.
- The class file as the Rust reader read it: flags, stack maps, and exception tables.
- The JVM verifier still reads each class that an `expect-run` example loads.
- The load order of modules, and the checks of the Rust catalogue of diagnostic codes.
- L0601 and the placements of L0602, which only `bux test` gives, and an example states `check`.
- `check` and `api` on the modules of `compiler/`, which change with each compiler edit.
- An empty file as an example, because an empty example teaches nothing.
- Every program under `tests/spec` run under stage 2 and under the launcher, beside the Rust run.
- A test runs each `expect-run` example on stage 1, and `bin/bootstrap` compares stage 2.
- A check that each code a phase raises has an explanation file.

The pre-commit hook runs `bin/bootstrap`, then `bin/bux test tests`, then `mycs check`.
It stops at the first that fails, and each must end with status 0.

---

## 8. Diagnostics

Compiler errors should be a major design priority.

Prefer:

```text
error: expected UserId, found Int

  12 | load_user(42)
     |           ^^

help: construct a UserId explicitly:

    load_user(UserId(42))
```

over cryptic JVM-oriented errors.

The user should never need to understand JVM bytecode to understand a compiler error.

This section sets the voice.
`docs/specs/diagnostics.md` fixes the exact layout, the codes, and the exit codes.

---

## 9. Initial language scope

The first version should deliberately be small.

### Version 0.1

Support:

* integers
* booleans
* strings
* functions
* local variables
* `if`, `for`, `break`, `continue`, `return`
* records
* lists, written and walked
* enums / ADTs
* pattern matching
* `Option`
* `Result`
* generics
* type inference
* canonical formatting as a compile gate, sequence included
* basic modules
* JVM bytecode generation

Do **not** implement initially:

* typeclasses
* effects
* concurrency
* advanced Java interop
* macros
* advanced type-level programming

Get the core type system correct first.

---

## 10. Version 0.2

Add:

* typeclasses
* traits/constraints
* derived implementations
* every operator a trait method, with the `Int` and `String` instances moved into the library
* a whole-number literal typed by what its context expects, refused where it does not fit
* a type a module declares reachable from the module that imports it
* collections
* richer pattern matching
* Java interop
* standard library
* package management

---

## 11. Version 0.3

Version 0.3 is self-hosting, and each item below is one the Bux-written compiler cannot do without.
`TODOS.md` holds the items in the order they are picked, and the order here is that order.

Add, in the language and the library:

* a list that grows and is read at an index
* a string read one code unit at a time and cut into a substring
* a program that takes arguments, exits with a code, and writes to standard error
* a file system surface: list, make, and delete a directory, and write a file
* a process started and its output read, which is how `bux run` starts `java`
* a library generic used at a type the program declares
* an instance written over a generic type, and `derive` reaching through a `List`
* a hashed map and set
* an instance reached from the module that imports its type
* a list that grows in amortized constant time, held as a buffer and a length

Then, the compiler phase by phase, lexer first, each checked against the Rust one on `tests/spec`.
Then the bootstrap of section 6, and the deletion of the Rust crates.

---

## 12. Version 0.4+

Concurrency comes first, and its first user is the compiler with its checks, which it makes fast.
That is `TODOS.md` Item 091, and the compiler dogfoods each part before any other program.
The second dogfood program is `1brc/src/main.bx`, the One Billion Row Challenge.
It reads one file with one process for each processor, over `library/` alone.
`docs/specs/billion-rows.md` states it, and section 7 records its wall time.

Add, once the compiler is Bux:

* concurrency: `process` with its enforced shape, `spawn`, a handle, a send deadline, cancellation
* JVM virtual-thread integration
* HTTP
* JSON
* database support
* a native binary via GraalVM native-image, once GraalVM tracks JDK 28 and Valhalla

No version adds a function value or a closure, as `docs/design.md` sections 11 and 14 state.

A type of the language written in the language is of interest for every type, `String` first.
Today `java.lang.String` carries a `String`, and it is the one Java class under a Bux value.
The prelude writes `Eq<String>` as Bux code, but the lowering puts `String.equals` under it.
So that logic lives in the JVM lowering, and a second target writes it again.
A `bux.String` value class over a `byte[]` moves it into the library, written once in Bux.
Then a second target lowers a machine word, a truth value, an array, and each `extern`, and no more.
The same holds for `Int` and `Bool` over the words a target has, and for any type after them.
The cost is measured first: the intrinsics of `String`, and a conversion at each `extern` with text.
`docs/design.md` section 3 already holds a prelude type to what a declared type can do.
So the language changes nothing, and only what carries a value moves.

### A native target

The JVM stays the one target, and a native binary is a question this section records, not a plan.
`docs/design.md` section 1 lists what the JVM gives: a GC, a JIT, cheap blocking, and platform APIs.
A native target must supply each of the four, and the runtime is the cost, not the code generator.
Go as a target supplies all four, but a Go lowering stays for good or goes whole.
No own code generator can link Go's runtime, so Go is not a step toward an own backend.
An own backend with an own runtime is the one end state with no foreign runtime, and Go took it.
Bux makes that cheaper than it sounds: no closure, no identity, and no state two processes share.
A moving GC is safe, because no program can observe an address.
Both end states need the same preparation, which Items 126 to 128 file, and each pays on the JVM.
The decision waits for a requirement, as every mechanism does, and native-image is the cheap answer.

Investigate:

* explicit effects
* effect inference
* richer record types
* anonymous structural records
* improved Java interop
* a `String`, and after it every prelude type, written in Bux over what a target has
* a native target, Go as a permanent backend or an own backend with an own runtime, as stated above
* compiler optimizations
* incremental compilation
* language server
* debugger integration

