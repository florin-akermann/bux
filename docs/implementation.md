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

The compiler should emit JVM bytecode directly or through a suitable intermediate representation.

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

The language's semantics must not be defined in terms of Java's type system.

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

Project Valhalla/value types should be considered when targeting modern JVMs.

---

## 3. Java interoperability

Java interoperability is important because the JVM ecosystem is valuable.

However, Java APIs should have a clear boundary.

For example:

```text
extern java class KafkaProducer<K, V> {
    ...
}
```

The standard library should provide idiomatic wrappers around common Java APIs.
Users should not be forced to interact directly with Java's object model.

The goal is:

```text
http.get(url)
```

rather than:

```text
java.net.http.HttpClient
    .newBuilder()
    ...
```

Java interop should be powerful but should not determine the design of the language.

---

## 4. Standard library

The standard library is small, and its data structures are smaller still.
The fewer methods a type has, the better; a method earns its place, and a `for` loop does the rest.
A method lands only where a plain loop over what the type already exposes cannot write it.
A map has no iterator, and the same holds for every convenience a loop already writes.
`docs/principles.md` question 12 is what a proposed method answers.

A data structure the library holds has the best asymptotic cost known for what it does.
A map looks a key up in constant time, and a list is read at an index in constant time.
The implementation is a textbook one, written as a school project writes it: plain and correct.
Tuning beyond that cost is refused until a measurement on a real program asks for it.

Initial areas:

```text
String
Collections
Option
Result
IO
Files
HTTP
Time
JSON
Concurrency
```

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
The launcher `bin/bux` starts them with `compiler/target/` on the class path.
`compiler/` and the directory above it are on the class path too, only for resources.
`compiler/help/` and `compiler/explanations/` hold the help text and the long form of each code.
The directory above holds `library/`, and the compiler reads all three as resources.
A module there has a `.bx` name, and the module loader reads no other extension.

What remains of the repository is small, and a JDK is the one tool it needs:

```text
bin/        bootstrap, bux, runner, and the seed seed.jar
compiler/   the compiler, its help text, and its explanations
library/    the standard library, read by the compiler as a resource
tests/      the runner, and tests/spec, the executable examples
example/    the example program
docs/       the specification, this document, and the behaviour specs
```

### The bootstrap

The bootstrap has a seed and two stages, and `bin/bootstrap` runs them.
The seed `bin/seed.jar` holds the classes of a compiler that an earlier compiler built.
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
```

rather than one mutable AST that means different things at different stages.

---

## 7. Testing strategy

Compiler development is specification-driven, and the executable examples are the specification.
`docs/specs/executable-examples.md` says what an example file is and what it expects.
An example is a `.bx` file under `tests/spec/<area>/`, and an area exists when an example needs it.

### The runner

`tests/runner.bx` is the runner, a Bux program, and `bin/runner` builds it and starts it.
It starts from the root of the repository and holds the compiler in one JVM.
It calls the functions of the compiler, so it starts no compiler process for each file.
It holds these parts, each in a module of its own under `tests/`:

- `exemplified.bx`: every example under `tests/spec`, to its header and to canonical form.
- `siblings.bx`: every sibling file, to the view of its phase: tokens, tree, surface, format.
- `documented.bx`: every `// example:` line of `tests/spec`, `library/`, `compiler/`, and `tests/`.
- The property modules that `every_property_held` in `runner.bx` names, one for each phase.
- `commanded.bx`: the command line, held to the golden answers under `tests/commands/`.
- `launched.bx`: the launcher `bin/bux`, and the time limit that a started program has.

The runner splits the parts into jobs, and one pool process hands each job to a worker process.
There is one worker for each processor that `Runtime.availableProcessors` gives.
The example lines and the examples are each split into at most one job for each worker.
Each golden file is one job, and each other check of the command line is one job.
The properties are one job, and the siblings are one job.
A worker sends the answer of its job to the pool, which holds each answer by the number of the job.
When every worker has left, the pool ends, and the runner counts the parts from its last state.
The runner waits for each worker before the pool, so a JVM error in a worker stops the runner.
It counts them in the order of the jobs, so no line depends on which worker ended first.

The runner starts a JVM for an example headed `expect-run` and for a command that runs one.
It starts one more JVM for each job of example lines, which runs the example lines of that job.
When the pool has ended, the runner writes one line for each part: the name, the count, and seconds.
The seconds are the sum of the times of the jobs of the part, which can be more than the wall time.
An example of such a line is `examples: 332 held in 4 s`.
The parts are examples, siblings, example lines, properties, and command lines, in that order.
Then it writes one line for each check it skipped and for each failure, then one line of counts.
It ends with status 0 only when nothing failed.

Each job of example lines holds one memo of typed modules, `command.Memo`, from call to call.
The memo holds each module by the canonical path of its file, so each module is typed once a job.
It also holds each module that `whole` found whole, and `whole` is not run on it again.
A command starts with an empty memo, so no answer of the command line depends on one.
`documented.bx` writes the run of each module, as `bux test` writes it, into a directory.
Each run gets a directory of its own, inside one directory made for the runs of its job.
Then the job starts `tests/examined.bx` once, and that program calls the `main` of each run.
Each run gets a class loader of its own, because two runs can hold one class name for two classes.
Before each run, `examined.bx` writes a line that names it, so a failure names its module.
A run that stops with an error, such as a stack overflow, is a failure, and the next run starts.
A run past the limit is a failure, and `examined.bx` starts again with the runs after it.
A module whose run cannot be built is reported as `bux test` reports it, and the others run.

`walk.ended` starts each program that the runner starts, and it gives each one a time limit.
`start_limit` in `runner.bx` is the one limit, in seconds, and every started program gets it.
A program that is still running at the limit is stopped, and it is a failure that names its check.
The runner then continues with the next check, so one program that never ends cannot stop it.
The limit is 60 s, because it guards against a hang, and not against a slow program.
The longest program is `examined.bx`, which took 3.9 s on 2026-09-23 for 146 runs.
It runs the examples of all modules, so its time grows with the modules and with machine load.
Two runners often run at the same time, so a limit near 3.9 s would stop a correct run.
Before Item 094, the longest program took 0.14 s, and the limit was 1 s.
`launched.bx` holds the mechanism: a program that never ends is stopped at a limit of 1 s.
`bin/runner golden` writes each golden file again, for an answer that changes on purpose.
On 2026-09-23 `bin/runner` took 182 s before Item 093 and 97 s after it.
On 2026-09-23 `bin/runner` took 98 s before Item 094 and 53 s after it.
Before Item 094 the examples took 5 s, and after it they took 4 s.
Before Item 094 the siblings took 0 s, and after it they took 0 s.
Before Item 094 the example lines took 78 s, and after it they took 34 s.
Before Item 094 the properties took 5 s, and after it they took 5 s.
Before Item 094 the command lines took 8 s, and after it they took 8 s.
On 2026-09-24 `bin/runner` took 64 s before Item 091 and 31 s after it.
On 2026-09-24 `bin/bootstrap` took 5.6 s before Item 091 and 5.8 s after it.
Item 091 did not put the compiler on processes, because a measurement showed no gain.
A build of the compiler wrote its 1400 classes in 0.31 s on one thread and on 1400 processes.
The processes spent four times the processor time, most likely in code the JIT had not compiled.

### Drawn properties

A property is a Bux function that tries one invariant on drawn input.
`tests/drawn.bx` is the generator, a linear congruential generator seeded by the case number.
The runner tries each property on 100 cases, so two runs draw the same input.
A failure names the property, the seed, and the drawn input, so a reader can try that case again.
Round trips come first: print then parse, a format that is idempotent, spans that cover the input.

### What the Rust tests held that the runner does not

Item 087 moved every Rust test to the runner, and it states each loss here.

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
- The runner runs each `expect-run` example on stage 1, and `bin/bootstrap` compares stage 2.
- A check that each code a phase raises has an explanation file.

The pre-commit hook runs `bin/bootstrap` and then `bin/runner`, and both must end with status 0.

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

* a function passed as a value; version 0.1 reaches a function by calling it
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

Add, once the compiler is Bux:

* concurrency: `process` with its enforced shape, `spawn`, a handle, a send deadline, cancellation
* JVM virtual-thread integration
* HTTP
* JSON
* database support
* a native binary via GraalVM native-image, once GraalVM tracks JDK 28 and Valhalla

Investigate:

* explicit effects
* effect inference
* richer record types
* anonymous structural records
* improved Java interop
* compiler optimizations
* incremental compilation
* language server
* debugger integration

