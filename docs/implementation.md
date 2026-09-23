# Lumen: Implementation and Roadmap

The language is specified in `docs/design.md`; this document covers how it is built and shipped.

## 1. JVM target

The JVM is the initial and primary runtime target, and Valhalla is what Lumen is built on.

**JDK 28 or later is targeted**, early access until it ships.
The emitted class-file version is 72, and no older JVM is supported or tested.
Every class the compiler writes is a value class, which JDK 28 holds in preview.
That preview is JEP 401, Value Classes and Objects, with JEP 539, Strict Field Initialization.
Every field the compiler writes is strict, which JEP 401 asks of every value class.
A preview class file carries minor version 65535, and a JVM loads one only when told to.
`lumen run` starts the JVM with `--enable-preview`, so a run needs nothing the reader must know.
Modern JVM features are used freely: invokedynamic, records, sealed classes, value classes.
A release of the JDK moves the target with it; there is no compatibility matrix and never will be.

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
lumen
```

Commands:

```text
lumen build
lumen run
lumen test
lumen fmt
lumen check
lumen api
lumen repl
lumen add
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

The compiler is written in Bux, and the Rust compiler is the bootstrap that gets it there.
Self-hosting comes before every feature the self-hosted compiler does not need.
Section 11 is the ladder of items it needs, and an item on the ladder is picked before one off it.

The self-hosted compiler runs on the JVM under `--enable-preview`, started by a `bux` launcher.
No native binary is needed to compile the compiler with itself.
GraalVM native-image waits until GraalVM tracks JDK 28 and Valhalla, and section 12 holds it.

The Bux compiler lives in `compiler/`, one module for each phase, and the lexer is the first.
A module there has a `.lm` name for now, because the module loader reads no other extension.
A harness in `crates/cli` holds each phase to the answer of the Rust phase it replaces.

The bootstrap has three stages.
Stage 0 is the Rust `bux`, which compiles the Bux-written compiler to stage 1.
Stage 1 compiles the same source to stage 2.
The Rust crates are deleted when stage 2 equals stage 1 byte for byte and both pass `tests/spec`.
Until then, the Rust compiler is the one that ships, and the crate structure below is its shape.

Crate structure:

```text
crates/
    lexer/
    parser/
    ast/
    format/
    modules/
    resolver/
    types/
    exhaustiveness/
    holes/
    hir/
    ir/
    jvm/
    api/
    diagnostics/
    cli/
```

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

Compiler development should be heavily specification-driven.

`docs/specs/executable-examples.md` says what an example file is and what it expects.

Tests should cover:

```text
tests/spec/
    lexer/
    parser/
    format/
    name_resolution/
    type_inference/
    generics/
    typeclasses/
    pattern_matching/
    exhaustiveness/
    errors/
    codegen/
    runtime/
    java_interop/
```

Use executable examples as language specifications.

For example:

```text
// identity.lm

fn identity(x) {
    x
}

assert(identity(42) == 42)
assert(identity("hello") == "hello")
```

And compile-fail tests:

```text
// mismatched_types.lm

type UserId = UserId(Int)

fn load(id: UserId) {
    ...
}

load(42)
```

The compiler must say that `Int` cannot be used where `UserId` is expected.

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
* a call written with its first argument in front, `maybe.or(fallback)` for `or(maybe, fallback)`
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

