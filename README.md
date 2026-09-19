# Lumen

> Go's simplicity.
> Haskell's types.
> The JVM's runtime.
> A Rust compiler.

Lumen is a small, statically typed language for practical software on the JVM.
Its compiler is written in Rust and emits JVM bytecode.

Three things set the everyday code apart:

- Source that is not in canonical form does not compile; `lumen fmt` produces that form.
- There are no anonymous functions; every function has a name, and names are first-class values.
- Control flow is Go's: `if`, `for`, `match`, `break`, `continue`, `return`.

The language is specified in `docs/design.md`.
The compiler and roadmap are in `docs/implementation.md`; the principles in `docs/principles.md`.
Only the current JDK release is targeted.
Agent and contributor guidelines are in `AGENTS.md`; the backlog is `TODOS.md`.

## Prerequisites

- Rust stable, with `cargo-nextest` and `cargo-audit` installed
- `mycs` on the `PATH` for the pre-commit sweep
- A JDK, only to run compiled programs; every compiler phase is tested without one

## Build

```sh
cargo build
cargo nextest run
git config core.hooksPath .githooks
```
