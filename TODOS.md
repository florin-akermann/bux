# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟡 Item 009: Lowering and JVM bytecode emission
**Depends on:** Item 008 — only checked programs are lowered.
`ir` and `jvm` crates lower the typed AST to a JVM IR and write class files in plain Rust.
Records and ADT variants become final classes; functions become static methods.
The class-file version is the current JDK's; no older JVM is supported.
Output is byte-reproducible: nothing depends on iteration order, a timestamp, or a build path.
[009][a] - Spec first in `docs/specs/codegen.md`: the class layout and the calling convention.
[009][b] - A class-file writer with a constant pool and stack-map frames, test-first per shape.
[009][c] - Lowering of each construct, test-first, verified with a class-file reader in tests.
[009][d] - `lumen build <file>` writes the class files; help topic added.
[009][e] - A property test: compiling the same source twice gives byte-identical class files.

## 🔴 Item 010: Running programs
**Depends on:** Item 009 — there must be bytecode to run.
`lumen run <file>` builds and runs on the JDK found via `JAVA_HOME`; missing JDK is a clear error.
[010][a] - Spec first in `docs/specs/run.md`: JDK discovery, exit codes, and the missing-JDK text.
[010][b] - `lumen run`, test-first; run-time executable examples are enabled in the harness.
[010][c] - The harness skips run-time examples with a named reason when no JDK is present.

## 🔴 Item 011: Canonical order as part of the compile gate
**Depends on:** Item 003, Item 008 — order is a formatter check, and arm order needs the variants.
Canonical form covers sequence, not only whitespace.
Imports sort, top-level declarations follow one order, and a definition precedes its use.
A `match` lists its arms in the order the variants are declared.
A new arm then has exactly one legal position, so no diff is ever reorder-only.
[011][a] - `docs/design.md` section 13 states the order rules before any of them is enforced.
[011][b] - Import and declaration order, and definition before use, checked by the formatter.
[011][c] - Arm order checked where the variant list is known, beside exhaustiveness, test-first.
[011][d] - Executable examples under `tests/spec/format/` and `tests/spec/exhaustiveness/`.

## 🔴 Item 012: `todo` as an explicit hole
**Depends on:** Item 007, Item 009 — a hole needs a type, and `lumen build` is what refuses it.
An unfinished body writes `todo("reason")`, which takes whatever type the context expects.
`lumen check` accepts a hole; `lumen build` refuses one, naming every hole and its reason.
Incompleteness is then greppable and gated, instead of filled in with plausible wrong code.
[012][a] - `docs/design.md` gains the hole; `docs/specs/holes.md` states the two behaviours.
[012][b] - Parsing and typing of `todo`, test-first; a hole unifies with any expected type.
[012][c] - `lumen build` refuses a program containing a hole, through the diagnostics renderer.
[012][d] - Executable examples: one file passes `check`, and the same file fails `build`.

## 🔴 Item 013: Machine-applicable diagnostics
**Depends on:** Item 003, Item 004 — the first fixes are the formatter's own repairs.
`lumen check --json` writes diagnostics as data: code, message, byte span, and a fix where known.
A canonical-form deviation carries its replacement, so a tool applies the compiler's own edit.
Nothing downstream reformats a file by hand to satisfy the gate.
[013][a] - Spec first in `docs/specs/diagnostics.md`: the JSON shape and the edit representation.
[013][b] - The serialiser, test-first against golden JSON; spans are byte offsets into the source.
[013][c] - Formatting deviations emit the canonical replacement for their span.
[013][d] - Help topic for `--json` in `crates/cli/src/help/`.

## 🔴 Item 014: `lumen api` prints a module's public surface
**Depends on:** Item 007 — the surface is printed from the typed AST.
`lumen api <file>` prints the public names of a module with their types, and nothing else.
One page then replaces reading a file to learn a signature, and inferred types are shown.
[014][a] - Spec first in `docs/specs/api-surface.md`: what is public, the order, and the layout.
[014][b] - The printer over the typed AST, test-first; its output is itself in canonical form.
[014][c] - Help topic for `api` in `crates/cli/src/help/`.
[014][d] - Executable examples under `tests/spec/api/`.

## 🔴 Item 015: An executable example per public function
**Depends on:** Item 005, Item 010 — the harness runs the examples, and running them needs a JDK.
A public function carries at least one executable example, or the module does not compile.
The examples run with the test suite, so a signature never drifts from the evidence for it.
[015][a] - Spec first in `docs/specs/doc-examples.md`: the form, and the missing-example error.
[015][b] - Extraction from doc comments, test-first, with spans into the original file.
[015][c] - `lumen test` runs them; a failing example names the function it documents.
[015][d] - Executable examples: a public function without one fails to compile.

## 🔴 Item 016: Named arguments where a signature repeats a type
**Depends on:** Item 007 — the rule is checked once parameter types are known.
A structural smell a type system can make unrepresentable belongs in the language, not in a linter.
mycs reports a swappable pair of arguments after the fact; Lumen refuses the call instead.
When two parameters share a type, the call site names its arguments: `rename(from: a, to: b)`.
A swapped pair of same-typed arguments is then a compile error rather than a runtime surprise.
[016][a] - `docs/principles.md` gains the question; `docs/design.md` section 11 gains the rule.
[016][b] - Spec first in `docs/specs/arguments.md`: the named-argument form and the error text.
[016][c] - The check after inference, test-first; positional calls stay legal when all types differ.
[016][d] - Executable examples under `tests/spec/arguments/`.

## 🔴 Item 017: A parameter is never a bare `Bool`
**Depends on:** Item 016 — the rule shares its spec and its error voice with named arguments.
mycs lints `Flag Argument`; the language removes it by refusing a `Bool` parameter outright.
A two-variant ADT takes its place, so `open(ReadOnly)` reads where `open(true)` did not.
The project already prefers one sum type over a pair of flags; this makes it the only option.
[017][a] - `docs/design.md` section 11 states the rule; `docs/specs/arguments.md` gives the error.
[017][b] - The spec settles the one carve-out: a boolean operation over `Bool` is still writable.
[017][c] - The check after inference, test-first; the `help:` names the two-variant ADT to write.
[017][d] - Executable examples under `tests/spec/arguments/`, including the compile-fail case.

## 🔴 Item 018: A discarded value is a compile error
**Depends on:** Item 007 — knowing that a value is discarded needs its type.
mycs lints `Swallowed Exception`; Lumen has no exceptions, but a `Result` can still be dropped.
Every expression statement is of type `()`, so a dropped `Result` or `Option` cannot compile.
Discarding is written, never implied: `_ = f(x)` says the value is deliberately thrown away.
[018][a] - `docs/design.md` section 5 states the rule; `docs/specs/discarding.md` gives the error.
[018][b] - The check over the typed AST, test-first; `_ =` is the one way to discard a value.
[018][c] - Executable examples under `tests/spec/type_inference/`, including the compile-fail case.

## 🔴 Item 019: Canonical naming as part of canonical form
**Depends on:** Item 003, Item 007 — casing is a formatter rule; the predicate rule needs types.
mycs reports `Cryptic Public Identifier`, `Acronym Casing`, and `Boolean Predicate Prefix`.
Canonical form makes them errors: `snake_case` functions and `PascalCase` types.
An acronym is a word, so `UserId` is canonical and `UserID` does not compile.
A public name is a word rather than an initial, and a `Bool` function reads as a predicate.
[019][a] - `docs/design.md` section 13 lists the naming rules alongside the formatting rules.
[019][b] - Casing, acronym, and length rules, test-first; the `help:` prints the canonical spelling.
[019][c] - The predicate-prefix rule, test-first, over functions whose return type is `Bool`.
[019][d] - Executable examples under `tests/spec/format/`.

## 🔴 Item 020: The two rules lowering already assumes
**Depends on:** Item 007, Item 009 — lowering asserts both, and nothing refuses either today.
Lowering assumes an assignment names a name, and that a function is reached only by calling it.
Neither rule is written down and neither is refused, so a program can reach lowering and panic.
[020][a] - `docs/design.md` states both rules where assignment and functions are described.
[020][b] - `docs/specs/` gains the two refusals in prose, with the error each one prints.
[020][c] - The front end refuses an assignment whose target is not a name, test-first.
[020][d] - The front end refuses a function name used as anything but the callee, test-first.
[020][e] - Executable examples for both refusals, each naming its code on its first line.
