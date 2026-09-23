# AGENTS.md

Bux is a small ML-inspired language with Go-like syntax and tooling, compiled to the JVM in Bux.
`docs/design.md` is the language specification; a language change is a change there first.
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
- Plain loops are the default idiom; higher-order functions are library, not a second paradigm.
- **Concurrency is Erlang's**: all messages, one mailbox per process, one enforced `process` shape.
- **Self-hosting comes first.** Version 0.3 is what the Bux compiler needs, in the order needed.
- Version 0.4 starts with concurrency, which makes the compiler fast; a native binary comes later.
- **Dogfood early and often.** The compiler, its tests, and its tools are written in Bux.
- Dogfooding finds the features Bux lacks and its bottlenecks, and both cost less when found early.
- A new feature has its first user in the compiler or its tools; a slow one is a `TODOS.md` item.
- A missing feature it finds is judged by `docs/principles.md`: a real need, or a bigger surface?
- **JDK 28 or later is targeted**, early access until it ships; no older class-file version.

## Language principles

- **One way to write a thing.** A second spelling is a cost with no guarantee behind it.
- So there is no anonymous function, no ternary, and no compound assignment beyond `+=`.
- `+=` is kept because a `for` loop that totals is the everyday shape; it is a ceiling, not a start.
- **A rule holds for every type alike, or it is no rule.** No built-in type is special.
- So `Int`, `Bool`, and `String` are types a library could have declared, with the same powers.
- Boxing and the primitive/reference split live inside the compiler; no program can observe them.
- An operator is a function with other syntax; `main` is `fn main(arguments: List<String>) -> Int`.
- **Values, not objects.** A value has no identity, `hashCode`, or `toString`; it is its state.
- Equality is opt-in: `==` needs `Eq`, which a type derives, and compares what a value holds.
- **`Option` never carries `()`**: `Some(())` is nullability by another name; `Result<(), E>` stays.
- Every type is a Valhalla value class from day one: identity-free, null-free, equal by state.
- **Nothing panics, ever.** No operation is partial, and the type says so; there is no `unwrap`.
- The compiler is held to the same: a program it cannot compile gets a diagnostic, never a crash.
- **The JVM is the target, not the model.** Its object model and its constraints stay out.
- **Formatting is a compile error.** Source that is not in canonical form does not compile.
- **Non-goals**, never implemented, suggested, or planned: ownership, borrowing, lifetimes.
- Likewise inheritance, null, checked exceptions, macros, implicit runtime magic, Java's types.

## Library principles

- **The standard library is small, and the fewer methods a type has, the better.**
- A method earns its place only where a `for` loop cannot write it; the map iterator is the example.
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
- Run `mycs check` on a change before anything else, and ask CodeScene MCP rather than guess.
- Use only the approved CodeScene tools; `docs/codescene-mcp.md` is the allow and deny list.

## Architecture

- One module per phase under `compiler/`, per `docs/implementation.md` section 6.
- Each phase consumes one typed representation and produces the next, never one mutable AST.
- A module is created by the todo that gives it real content, never ahead of it.
- `compiler/main.lm` is the command line only; `compiler/command.lm` holds every command.
- `tests/spec/<area>/*.lm` are executable examples that are the language specification.
- `docs/specs/` holds behaviour specs written before a feature lands, updated in place.
- `bin/bootstrap` builds the compiler from the seed `bin/seed.jar`, and `bin/bux` starts it.

## Workflow

- Behaviour first: a `docs/specs/` spec states in prose what the change must observably do.
- Tests come early: plain test code that reads as the behaviour, then code to pass it.
- No Gherkin and no red/green micro-cycle; a behaviour, its tests, and its code land together.
- Every phase gets drawn properties (`tests/drawn.lm`) for its invariants; examples are not enough.
- Round trips are the first properties: print-then-parse, format idempotence, spans covering input.
- The runner `tests/runner.lm` holds every check, one module under `tests/` for each part.
- Shared checks go in `tests/held.lm` and `tests/walk.lm`, reached with `import`.
- **Tests never run git**, even in a temp dir; a subprocess git can corrupt the repo state.
- A check that starts a second JVM is skipped with a named reason when `JAVA_HOME` names none.
- `bin/bootstrap`, `bin/runner`, and the built-in `/code-review` pass before every commit.
- `TODOS.md` is the sole task tracker; there is no `gh` and no GitHub integration.
- A user-facing feature is documented under `lumen --help` in the todo that adds it.
- Help text lives in `compiler/help/*.md`, read as a class-path resource: one source, no drift.

## Tools and dependencies

- **Offline-first.** Every build and test works with no network; nothing is fetched or searched.
- **No tool installation.** The one runtime is a JDK, which the user installs and `JAVA_HOME` names.
- **Write it, don't import it.** The lexer, parser, and class-file writer are in-house.
- No dependency is added: the JDK is the one thing a build needs from outside the repository.
- Python scripts run with `uv run <script>.py`, never bare `python3`.

## Name

- **The language is Bux.** Lumen is the old name, and it survives only where nothing has moved yet.
- Every new name is the new one: a `bux` binary, `bux-*` packages, `.bx` sources, "Bux" in prose.
- There is no big-bang rename; a task moves what it touches and stops there.
- A tree-wide rename (a package, the binary, the extension) is one `TODOS.md` item, landed whole.

## Markdown Prose Style

- **One sentence per line, maximum 100 characters.** mycs holds every `.md` file to both.
- The rules are lifted only in `compiler/help/`, where terminal-wrapped help topics live.
- Never wrap a sentence across lines; shorten it, and split it only if it still will not fit.
- This section is itself the rule: `Governance Doc Omits Prose Style` fires if it goes missing.

## Commit messages

- Commit messages read `Item NNN: <what changed>`; `chore:` for housekeeping outside an item.
- Never name the AI agent in a commit message.
