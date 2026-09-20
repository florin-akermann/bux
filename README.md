# Lumen

> Go's simplicity.
> Haskell's types.
> Valhalla's values.
> A Rust compiler.

Lumen is a small, statically typed language for practical software, built on Valhalla.
Its compiler is written in Rust and emits JVM bytecode.

Four things set the everyday code apart:

- Source that is not in canonical form does not compile; `lumen fmt` produces that form.
- There are no anonymous functions; every function has a name, and names are first-class values.
- Control flow is Go's: `if`, `for`, `match`, `break`, `continue`, `return`.
- Values, not objects: nothing has identity, and `Int` is no more special than a type you declare.

The JVM is a target, not a model: none of its constraints is kept, its object model least of all.
Every type is a Valhalla value class from day one: identity-free, null-free, equal by state.

The language is specified in `docs/design.md`.
The compiler and roadmap are in `docs/implementation.md`; the principles in `docs/principles.md`.
JDK 28 or later is targeted, early access until it ships; that is where value classes live.
Agent and contributor guidelines are in `AGENTS.md`; the backlog is `TODOS.md`.

## Prerequisites

- Rust stable, with `cargo-nextest` and `cargo-audit` installed
- `mycs` on the `PATH` for the pre-commit sweep
- JDK 28 or later, only to run compiled programs; every compiler phase is tested without one
- Until JDK 28 ships, an early-access build of it; export `JAVA_HOME` as its `Contents/Home`

## Build

```sh
cargo build
cargo nextest run
git config core.hooksPath .githooks
```

## The example program

`example/main.lm` is everyday Lumen in one screen: a record, an ADT, a `match`, and a list walked.

```sh
cargo run --bin lumen -- run example/main.lm
```

It ends normally and exits `0`, which is the whole of what it shows.
The answer it works out is an `Int`, and version 0.1 has no way to write a number out.
`docs/specs/example-program.md` says what the directory holds and why that is enough.

Its public surface is one page, which is what a reader consults to learn a signature:

```sh
cargo run --bin lumen -- api example/main.lm
```
