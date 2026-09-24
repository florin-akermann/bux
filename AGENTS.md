# AGENTS.md

Bux is a small ML-inspired language with Go's philosophy and tooling, compiled to the JVM in Bux.
`docs/design.md` is the language specification; a language change is a change there first.
`docs/rationale.md` gives the reasons, and an agent opens it only to propose a language change.
`docs/implementation.md` says how the compiler is built and what ships when.
`docs/principles.md` holds the questions every proposed feature must answer.
AI agents write most of the code, and agents drift without a fixed anchor.
This file is that anchor, and it is written as principles.
A concrete rule below is one example of a principle applied, not the whole of it.
When a case is not covered, apply the principle; do not search for the nearest example.

## The one principle: as simple as possible, but no simpler

Everything else in this file is this sentence applied to a language, a compiler, or a workflow.
"As simple as possible" means the smallest design that meets an executable requirement of today.
"But no simpler" means correctness, typed failure, and proven performance keep what they need.
Both halves bind equally: a design is refused for doing too much and for doing too little.
An example of the first half: a map has no iterator, because a `for` loop over its keys is enough.
An example of the second half: `17 / 0` is `None`, because a panic is simpler and wrong.

## What Bux is

- **Goal**: Go's simplicity, Haskell's types, Erlang's messages, Valhalla's values, a Bux compiler.
- Everyday Bux code is mostly `for` loops, plus algebraic data types and `match`.
- Plain loops are the one idiom, and `docs/design.md` section 12 states it.
- **Concurrency is Erlang's**: all messages, as `docs/design.md` section 15 states.
- **Self-hosting comes first.** Version 0.3 is what the Bux compiler needs, in the order needed.
- Version 0.4 starts with concurrency, which makes the compiler fast; a native binary comes later.
- **Dogfood early and often.** The compiler, its tests, and its tools are written in Bux.
- Dogfooding finds the features Bux lacks and its bottlenecks, and both cost less when found early.
- A new feature has its first user in the compiler or its tools; a slow one is a `TODOS.md` item.
- A missing feature it finds is judged by `docs/principles.md`: a real need, or a bigger surface?
- **JDK 28 or later is targeted**, early access until it ships; no older class-file version.

## Language principles

`docs/design.md` states each rule once; a line below names a principle and points at its section.

- **One way to write a thing.** A second spelling is a cost with no guarantee behind it.
- `docs/design.md` section 2 states the sugar rule, and `+=` is the one shorthand it keeps.
- **A rule holds for every type alike, or it is no rule.** No built-in type is special.
- `docs/design.md` sections 2 and 3 state what that asks of `Int`, of an operator, and of `main`.
- **Values, not objects.** A value is its state; `docs/design.md` sections 2, 8, and 10 say how.
- **Nothing panics, ever.** No operation is partial; `docs/design.md` section 5 states the rule.
- `Option` never carries `()`, and `docs/design.md` section 5 states that rule too.
- The compiler is held to the same: a program it cannot compile gets a diagnostic, never a crash.
- **The JVM is the target, not the model.** `docs/design.md` section 2 states what stays out.
- **Formatting is a compile error.** `docs/design.md` section 13 states the canonical form.
- **Non-goals** are never implemented, suggested, or planned; `docs/design.md` section 2 lists them.

## Library principles

- **The standard library is small**; `docs/principles.md` question 12 is the test a function passes.
- A library data structure has the best known asymptotic cost, written plainly and never tuned.

## Simplicity in the compiler

- **Build the smallest design that meets today's executable requirements**, never anticipated ones.
- Machinery earns its keep with a concrete requirement or a failing example, never with "later".
- A measured performance result, a security boundary, or an operational constraint counts equally.
- So there is no registry, plugin protocol, pluggable strategy, retry, migration, or config knob.
- Every abstraction, type, state, config value, and branch protects an observable requirement.
- One that protects nothing is removed, or never introduced.
- **Delete and narrow** rather than rename or generalise; a removed concept does not return renamed.
- Prefer platform guarantees and direct composition over custom infrastructure.
- Treat disposable data as disposable; a cold start or a rebuild is often the answer.
- One shared policy beats per-operation tuning; vary behaviour only when an example proves it must.
- **"No simpler" binds equally.** Never weaken a test, invariant, or failure semantic to cut lines.
- Before a non-trivial mechanism, state the requirement it protects and the alternatives weighed.
- Executable examples are written around observable behaviour, never around machinery.

## Naming and abstraction

- **Wishful thinking first.** Write the code you wish existed, then name it so it reads plainly.
- **Parse, don't merely validate.** Turn boundary input into types that keep established facts.
- **Make illegal states unrepresentable.** Elegance leaves the invalid case unwriteable.
- Adequacy writes the invalid case and rejects it afterwards; a warranted type is held to elegance.
- Prefer domain types over `String`, `u32`, and `bool`, and one sum type over a pair of flags.
- Push fallibility to construction and decoding boundaries so interior functions can be total.
- Construction proves an invariant; a wrapper name or a comment asserts one, which is not proof.
- Add type ceremony only where it removes a partial operation, a repeated guard, or an ambiguity.
- A name describes role and intent, not type; no `Utils`, `Misc`, `Helper`, `get_`, or `set_`.

## Code health

- **Code Health is authoritative**, and the target is 10.0; 9+ is not good enough.
- A regression is refactored, not accepted; nothing is grandfathered, and the hook sweeps the tree.
- **Boy Scout Rule: a touched file leaves better** than it was found.
- A carve-out (`[ignore]`, `default_skip`, `[disable]`, a rule disabled in `mycs.toml`) is a debt.
- A debt carries its reason beside it; a bare suppression in source is refused.
- Run `mycs check` on a change where it is installed, and ask CodeScene MCP rather than guess.
- mycs is optional: the hook runs it where it is on the `PATH`, and proceeds where it is not.
- Use only the approved CodeScene tools; `docs/codescene-mcp.md` is the allow and deny list.

## Architecture

- One module per phase under `src/`, per `docs/implementation.md` section 6.
- Each phase consumes one typed representation and produces the next, never one mutable AST.
- A module is created by the todo that gives it real content, never ahead of it.
- `src/main.bx` is the command line only; `src/command.bx` holds every command.
- `tests/spec/<area>/*.bx` are executable examples that are the language specification.
- `docs/specs/` holds behaviour specs written before a feature lands, updated in place.
- `bin/bootstrap` builds the compiler from the seed `bin/seed.jar`, and `bin/bux` starts it.

## Workflow

- Behaviour first: a `docs/specs/` spec states in prose what the change must observably do.
- Tests come early: plain test code that reads as the behaviour, then code to pass it.
- No Gherkin and no red/green micro-cycle; a behaviour, its tests, and its code land together.
- Every phase gets drawn properties (`tests/drawn.bx`) for its invariants; examples are not enough.
- Round trips are the first properties: print-then-parse, format idempotence, spans covering input.
- Every check is a `test` block of a module under `tests/`, and `bin/bux test` runs them all.
- Shared checks go in `tests/held.bx` and `tests/walk.bx`, reached with `import`.
- **Tests never run git**, even in a temp dir; a subprocess git can corrupt the repo state.
- A check that starts a second JVM is skipped with a named reason when `JAVA_HOME` names none.
- `bin/bootstrap`, `bin/bux test`, and the built-in `/code-review` pass before every commit.
- `TODOS.md` is the sole task tracker; there is no `gh` and no GitHub integration.
- A user-facing feature is documented under `bux --help` in the todo that adds it.
- Help text lives in `help/*.md`, read as a class-path resource: one source, no drift.

## Tools and dependencies

- **Offline-first.** Every build and test works with no network; nothing is fetched or searched.
- **No tool installation.** The one runtime is a JDK, which the user installs and `JAVA_HOME` names.
- **Write it, don't import it.** The lexer, parser, and class-file writer are in-house.
- No dependency is added: the JDK is the one thing a build needs from outside the repository.
- Python scripts run with `uv run <script>.py`, never bare `python3`.

## Name

- **The language is Bux**: a `bux` command, `.bx` sources, and the JVM package `bux/`.
- "Bux" is the name in prose, in help text, in comments, and in every name a program sees.
- A tree-wide rename (a package, the binary, the extension) is one `TODOS.md` item, landed whole.

## Markdown Prose Style

- **One sentence per line, maximum 100 characters.** the hook holds every `.md` file to both.
- The rules are lifted only in `help/`, where terminal-wrapped help topics live.
- Never wrap a sentence across lines; shorten it, and split it only if it still will not fit.
- This section is itself the rule: `Governance Doc Omits Prose Style` fires if it goes missing.

## Commit messages

- Commit messages read `Item NNN: <what changed>`; `chore:` for housekeeping outside an item.
- Never name the AI agent in a commit message.
