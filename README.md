# Lumen

> Go's simplicity.
> Haskell's types.
> The JVM's runtime.
> A Rust compiler.

Lumen is a small, statically typed language for practical software; its first target is the JVM.
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
