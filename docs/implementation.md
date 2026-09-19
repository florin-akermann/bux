# Lumen: Implementation and Roadmap

The language is specified in `docs/design.md`; this document covers how it is built and shipped.

## 1. JVM target

The JVM is the initial and primary runtime target, and Valhalla is what Lumen is built on.

**JDK 28 or later is targeted**, early access until it ships.
The emitted class-file version is 72, and no older JVM is supported or tested.
Every class the compiler writes is a value class, which JDK 28 holds in preview.
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

The standard library should be relatively small and practical.

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

The compiler should be written in Rust.

Crate structure:

```text
crates/
    lexer/
    parser/
    ast/
    format/
    resolver/
    types/
    exhaustiveness/
    hir/
    ir/
    jvm/
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
* collections
* richer pattern matching
* Java interop
* standard library
* package management

---

## 11. Version 0.3

Add:

* concurrency
* channels
* JVM virtual-thread integration
* HTTP
* JSON
* database support

---

## 12. Version 0.4+

Investigate:

* explicit effects
* effect inference
* richer record types
* anonymous structural records
* JVM value types / Project Valhalla
* improved Java interop
* compiler optimizations
* incremental compilation
* language server
* debugger integration

