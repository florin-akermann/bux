# AGENTS.md

Lumen is a small ML-inspired language with Go-like syntax and tooling, compiled to the JVM in Rust.
`docs/design.md` is the language specification; a language change is a change there first.
`docs/implementation.md` says how the compiler is built and what ships when.
`docs/principles.md` holds the questions every proposed feature must answer.
AI agents write most of the code, and agents drift without a fixed anchor.
This file is that anchor; consistency is the throughline, and the bar is Code Health 10.0.

## Project Scope
- **Goal**: Go's simplicity, Haskell's types, the JVM's runtime, a Rust compiler.
- Everyday Lumen code reads like Go: basically a bunch of `for` loops, plus ADTs and `match`.
- **Formatting is a compile error.** Source that is not in canonical form does not compile.
- **No anonymous functions.** Every function has a name; functions are first-class by name.
- Plain loops are the default idiom; higher-order functions are library, not a second paradigm.
- **Non-goals** — never implement, suggest, or plan: ownership, borrowing, lifetimes, inheritance.
- Likewise null, checked exceptions, macros, implicit runtime magic, or Java's type system.
- The JVM is the implementation target, not the semantic model; no JVM detail leaks into Lumen.
- Version 0.1 is `docs/implementation.md` section 9; nothing from a later version lands earlier.
- **Only the current JDK release is targeted.** No older class-file version, no compatibility matrix.

## Agent TL;DR
- **Code Health is authoritative** — the single source of truth for maintainability.
- **Target Code Health 10.0.** The standard for AI-friendly code; 9+ is not "good enough."
- **Run `mycs check` on your changes first** — ahead of anything else.
- **The hook sweeps the whole tree on every commit** — a bare `mycs check`; nothing is grandfathered.
- A carve-out (`[ignore]`, `default_skip`, `[disable]`) is a debt needing a reason beside it.
- If Code Health regresses, **refactor — don't declare done.**
- **Boy Scout Rule: always leave the code cleaner than you found it**; a touched file leaves better.
- When in doubt, call the appropriate CodeScene MCP tool — don't guess.
- No `#[allow]` attributes in source; a lint is fixed, or allowed in `Cargo.toml` with its reason.

## Architecture
- A Cargo workspace; one crate per phase under `crates/`, per `docs/implementation.md` section 6.
- A crate is created by the todo that gives it real content, never ahead of it.
- Each phase consumes one typed representation and produces the next, never one mutable AST.
- `crates/cli/` — the `lumen` binary; argument parsing and help only, no compiler logic.
- `tests/spec/<area>/*.lm` — executable examples that are the language specification.
- `docs/specs/` — behaviour specs written before a feature lands; updated in place, never forked.
- A JDK is needed only to run compiled programs; every compiler phase is tested without one.

## TDD Workflow
- Red/green cycles; run `cargo nextest run` before every commit; spec, test-first, review, commit.
- Every phase gets property-based tests (`hegeltest`) for its invariants; examples are not enough.
- Round trips are the first properties: print-then-parse, format idempotence, spans covering input.
- Each crate has one integration-test binary, `tests/integration/main.rs`, declaring `mod` per file.
- Shared helpers go in `tests/integration/common.rs`, reached with `use crate::common;`.
- **Tests must never run git commands** — even in temp dirs; subprocess git can corrupt repo state.
- Tests never require a JDK; a run-time example is skipped with a named reason when none is present.

## Review Gate
- The built-in `/code-review` is required before commit (semantic gate); `/work` runs it at medium.
- The dev-team plugin's `/specs`, `/plan`, and `/build` are not part of the loop; implement directly.
- `review-config.json` trims the plugin's own `/code-review` to correctness and spec compliance.
- No `gh` CLI — no GitHub integration; `TODOS.md` is the sole task tracker.

## CodeScene MCP
- Use only the approved CodeScene tools; see `docs/codescene-mcp.md` for the allow/deny list.
- Never use socio-technical features, `get_config` / `set_config`, or the business-case tool.

## Tools & Dependencies
- **Offline-first.** Every build and test works with no network; never fetch data.
- **No web search.** Consult local files, man pages, `cargo doc`, and the code — not the web.
- **No tool installation.** No system packages, binaries, or package managers (brew, apt, etc.).
- The one runtime the project needs is a JDK, which the user installs and `JAVA_HOME` names.
- **Write it, don't import it.** Default answer to "add a crate?" is no — implement in plain Rust.
- The class-file writer, the lexer, and the parser are written in-house; there is no parser generator.
- Add a crate only when it cannot be replicated at acceptable cost, with `# approved:` beside it.
- Python scripts: `uv run <script>.py` — never bare `python3`.
- RustRover MCP beats textual tools for semantic operations; see `docs/rustrover-mcp.md`.

## CLI Documentation Rule
- Any user-facing feature **must** be documented under `lumen --help` in the todo that adds it.
- Help text lives in `crates/cli/src/help/*.md`, surfaced via `include_str!` — one source, no drift.

## Markdown Prose Style
- **One sentence per line, maximum 100 characters.** mycs holds every `.md` file to both.
- The rules are lifted only in `crates/cli/src/help/`, where terminal-wrapped help topics live.
- Never wrap a sentence across lines; shorten it, and split it only if it still will not fit.
- This section is itself the rule: `Governance Doc Omits Prose Style` fires if it goes missing.

## Simplicity and Proportionality
- **Build the smallest design that meets today's executable requirements**, never anticipated ones.
- **No smartness by default** — no speculative extensibility, framework, or pluggable strategy.
- Likewise no registry, plugin protocol, compatibility state, migration, retry, or extra config knob.
- Add such machinery only when a concrete requirement or a failing example proves it needed.
- A measured performance result, a security boundary, or an operational constraint proves it equally.
- Every abstraction, type, state, config value, and branch must earn its keep.
- One that protects no observable requirement is removed, or never introduced.
- Prefer ordinary platform guarantees and direct composition over custom infrastructure.
- That means explicit constructors and plain control flow over machinery that assembles them for you.
- **Treat disposable data as disposable.** A cold start or a rebuild is often the answer.
- One shared policy beats per-operation tuning; vary behaviour only when an example proves it must.
- **Delete and narrow** rather than rename or generalise accidental complexity.
- Never reintroduce a removed concept under a different name.
- Keep cross-cutting concerns at application boundaries and composition roots.
- **"No simpler" binds equally.** Correctness, typed failures, and proven performance keep theirs.
- Never weaken a test, invariant, or failure semantic to cut line or type count.
- Before a non-trivial mechanism, state the requirement it protects and the alternatives weighed.
- If the only justification is "might need it later", don't build it.
- Write executable examples around observable behaviour, never around implementation machinery.

## Naming & Abstraction
- **Wishful thinking first.** Write the code you wish existed, then name so it reads plainly.
- **Parse, don't merely validate.** Turn boundary input into types that preserve established facts.
- **Make illegal states unrepresentable.** A warranted type is held to elegance, not adequacy.
- Elegance leaves the invalid case unwriteable; adequacy writes it and rejects it afterwards.
- Prefer domain types over `String`, `u32`, and `bool`, and one sum type over a pair of flags.
- Push fallibility to construction and decoding boundaries so interior functions can be total.
- Keep unsafe constructors private and make mutation preserve each invariant.
- A named wrapper is not proof: construction must prove the invariant, not assert it in a comment.
- Add type ceremony only when it removes partial operations, repeated guards, or ambiguous primitives.
- **Parameter names describe role and intent**, not just type; let scope determine name length.
- **No dumping-ground names** (`Utils`, `Misc`, `Helper`); avoid noisy `get_`/`set_` prefixes.

## Commit Message Rules
- Commit messages read `Item NNN: <what changed>`; `chore:` for housekeeping outside an item.
- Never name the AI agent in a commit message, nor add a `Co-Authored-By` trailer naming an AI tool.
