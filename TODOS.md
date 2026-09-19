# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 001: Lexer for the version 0.1 surface
A `lexer` crate turns source text into tokens with byte spans, for the version 0.1 surface.
Tokens: identifiers, keywords, integer and string literals, operators, punctuation, and `:=`.
An unknown character is a token in its own right, so the lexer never fails; the parser reports it.
[001][a] - Spec first in `docs/specs/lexer.md`: the token set, spans, and the bad-character token.
[001][b] - The token type and `lex(source) -> Vec<Token>`, test-first per token class.
[001][c] - Property tests: spans are contiguous and cover the input; lexing is total on any text.
[001][d] - Executable examples under `tests/spec/lexer/`.

## 🔴 Item 002: Parser and untyped AST
**Depends on:** Item 001 — the parser consumes its tokens.
`ast` and `parser` crates: functions, `:=` bindings, `var`, `if`, `for`, `match`, records, ADTs.
The AST is untyped and immutable; every later phase produces a new representation.
There are no anonymous functions in the grammar; a function is named or it is a parse error.
[002][a] - Spec first in `docs/specs/grammar.md`: the grammar, and the parse-error voice.
[002][b] - Recursive-descent parser, test-first per production, with span-carrying errors.
[002][c] - Property test: a printed AST parses back to an equal AST (with Item 003's printer).
[002][d] - Executable examples under `tests/spec/parser/`, including parse-fail examples.

## 🔴 Item 003: Canonical formatting as a compile gate
**Depends on:** Item 002 — the formatter is the AST pretty-printer.
A `format` crate prints the one canonical form of an AST; `docs/design.md` section 13 is the rule.
A source file compiles only if `format(parse(source)) == source`, byte for byte.
[003][a] - Spec first in `docs/specs/formatting.md`: indentation, spacing, line breaks, comments.
[003][b] - The printer, test-first per construct; comments survive the round trip.
[003][c] - Property tests: the printer is idempotent, and its output always parses.
[003][d] - `lumen fmt <file>` rewrites in place; `lumen check <file>` reports the first deviation.
[003][e] - Help topics for `fmt` and `check` in `crates/cli/src/help/`.

## 🔴 Item 004: Diagnostics rendering
**Depends on:** Item 002 — the first diagnostics are parse errors.
A `diagnostics` crate renders `error: <message>`, the source line, a caret span, and a `help:`.
The voice is `docs/implementation.md` section 8; no diagnostic ever mentions the JVM.
[004][a] - Spec first in `docs/specs/diagnostics.md`: the layout and the exit code.
[004][b] - Rendering, test-first, against golden text for one-line and multi-line spans.
[004][c] - Parse and formatting errors flow through it from `lumen check`.

## 🔴 Item 005: Executable-example harness
**Depends on:** Item 004 — a compile-fail example asserts on rendered diagnostics.
`tests/spec/<area>/*.lm` files are the language specification (`docs/implementation.md` section 7).
A `// expect-error: <text>` header marks a compile-fail example; the rest must compile.
[005][a] - Spec first in `docs/specs/executable-examples.md`: file layout and expectation syntax.
[005][b] - One test binary walks `tests/spec/`, runs each file, and names the failing example.
[005][c] - Run-time examples are skipped with a named reason until Item 010 lands.

## 🔴 Item 006: Name resolution and basic modules
**Depends on:** Item 002 — resolution consumes the untyped AST.
A `resolver` crate produces a `ResolvedAst` where every name points at its definition.
One file is one module; `import` brings another module's public names into scope.
[006][a] - Spec first in `docs/specs/modules.md`: visibility, imports, the unresolved-name error.
[006][b] - Scopes and definitions, test-first; shadowing rules match Go's.
[006][c] - Executable examples under `tests/spec/name_resolution/`.

## 🔴 Item 007: Type inference
**Depends on:** Item 006 — inference runs on the resolved AST.
A `types` crate does Hindley-Milner inference with let-polymorphism over the version 0.1 types.
`Option` and `Result` are ordinary ADTs in a prelude; `?` desugars to a match on `Result`.
A newtype such as `UserId(Int64)` is never unified with its representation.
[007][a] - Spec first in `docs/specs/types.md`: the types, generalisation, and the mismatch error.
[007][b] - Unification and generalisation, test-first, producing a `TypedAst`.
[007][c] - Records, ADTs, generics, and `?`, each test-first.
[007][d] - Property tests: inferred types are principal on generated well-typed terms.
[007][e] - Executable examples under `tests/spec/type_inference/` and `tests/spec/generics/`.

## 🔴 Item 008: Exhaustiveness checking
**Depends on:** Item 007 — patterns are checked against their inferred types.
An `exhaustiveness` crate rejects a non-exhaustive `match` and names the missing patterns.
[008][a] - Spec first in `docs/specs/exhaustiveness.md`: the algorithm and the error text.
[008][b] - The usefulness algorithm, test-first, over ADTs, records, and literals.
[008][c] - Executable examples under `tests/spec/exhaustiveness/`.

## 🔴 Item 009: Lowering and JVM bytecode emission
**Depends on:** Item 008 — only checked programs are lowered.
`ir` and `jvm` crates lower the typed AST to a JVM IR and write class files in plain Rust.
Records and ADT variants become final classes; functions become static methods.
The class-file version is the current JDK's; no older JVM is supported.
[009][a] - Spec first in `docs/specs/codegen.md`: the class layout and the calling convention.
[009][b] - A class-file writer with a constant pool and stack-map frames, test-first per shape.
[009][c] - Lowering of each construct, test-first, verified with a class-file reader in tests.
[009][d] - `lumen build <file>` writes the class files; help topic added.

## 🔴 Item 010: Running programs
**Depends on:** Item 009 — there must be bytecode to run.
`lumen run <file>` builds and runs on the JDK found via `JAVA_HOME`; missing JDK is a clear error.
[010][a] - Spec first in `docs/specs/run.md`: JDK discovery, exit codes, and the missing-JDK text.
[010][b] - `lumen run`, test-first; run-time executable examples are enabled in the harness.
[010][c] - The harness skips run-time examples with a named reason when no JDK is present.
