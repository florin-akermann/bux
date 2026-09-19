# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟡 Item 011: Canonical order as part of the compile gate
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

## 🔴 Item 021: `example/` holds a runnable program
A newcomer's first Lumen program lives at `example/`, and `lumen run example/main.lm` runs it.
The tree holds spec files and compiler tests; none of them is a program somebody would write.
This one is everyday Lumen: a record, an ADT, a `match`, and a `for` loop, read in one screen.
There is no printing yet, so the program is observed by ending normally rather than by output.
A test runs it with the suite, so the one program the README promises can never rot.
[021][a] - `docs/specs/example-program.md` states what the directory holds and what running it does.
[021][b] - `example/main.lm` itself, in canonical form and held by `lumen check` like any source.
[021][c] - An integration test runs it through the CLI, skipped with a named reason when no JDK.
[021][d] - README names the one command that runs it; class files beside it are ignored by git.

## 🔴 Item 022: `==` requires `Eq` rather than every type
Every type is equatable today, which is Java's object model arriving by the back door.
`crates/types/src/infer.rs` accepts `==` between any two operands that share a type.
`crates/ir/src/lower/classes.rs` gives every record and every variant a generated `equals`.
`==` is `Eq`, and no type is equatable without an instance; a built-in type is no exception.
Version 0.1 has no `derive`, so the library ships the three instances: `Int`, `Bool`, `String`.
`==` on a record is then an error naming `Eq`, which 0.2 lets any type derive the same way.
Widening later costs nothing; withdrawing a universal `==` once programs rely on it costs plenty.
[022][a] - `docs/design.md` section 8 states that `==` needs `Eq`, and lists the 0.1 instances.
[022][b] - `docs/specs/codegen.md` loses the paragraph putting `equals` on every class.
[022][c] - Inference refuses `==` without an `Eq` instance, test-first, under its own code.
[022][d] - Lowering stops generating `equals`; a literal pattern still compares the three types.
[022][e] - Executable examples under `tests/spec/type_inference/`, including the compile-fail case.

## 🔴 Item 023: Identity is never observable
**Depends on:** Item 022, Item 024 — the rule lands in the section Item 024 writes.
Nothing in Lumen asks whether two values are one object, and nothing ever should.
The JVM gives every object a header, `hashCode`, `getClass`, and `toString`, which Lumen hides.
A generated class still inherits `Object.equals`, so dropping the generated one leaves identity.
Writing the rule down and guarding it stops a later phase reaching for an inherited method.
[023][a] - `docs/design.md` section 2 lists identity among the non-goals, beside the object model.
[023][b] - `docs/specs/codegen.md` states that no generated class declares an inherited method.
[023][c] - A test over the lowered classes: none declares `hashCode`, `getClass`, or `toString`.
[023][d] - A test that no lowering emits a reference comparison, so identity has no opcode either.

## 🔴 Item 024: The JVM is a target, not a model
The JVM is where Lumen compiles first, and that is the whole of its authority over the language.
`AGENTS.md` and the README state it; `docs/design.md`, which they cite, has one buried clause.
`docs/principles.md` asks it as a question, so each JVM habit still gets argued from scratch.
The object model is the standing instance: identity, `equals`, `hashCode`, and a root class.
The eight special primitives are the other: in Lumen no built-in type is special.
Stated once as a rule, every later question about a JVM habit is settled by citing it.
[024][a] - `docs/design.md` section 2 gains the rule as prose, not as one more item in a list.
[024][b] - The rule names what Lumen declines: the object model, boxing, and special primitives.
[024][c] - The rule states the model adopted instead: value semantics, in Valhalla's shape.
[024][d] - `docs/principles.md` question 6 points at the rule rather than restating it.

## 🔴 Item 025: Generics are specialized, never erased
`docs/specs/codegen.md` erases a type parameter to `java.lang.Object`, boxing an `Int` across it.
That is the one place a program can tell `Int` from a declared type, and the one place it boxes.
A generic function is instead lowered once per instantiation, with the descriptor its types give.
An `Int` then crosses a generic as a `long`, and a record as itself, so no type is special.
The cost is one method per instantiation and a generic body that must reach the module using it.
[025][a] - `docs/design.md` section 7 states that a generic is specialized at each use.
[025][b] - `docs/specs/codegen.md` replaces erasure and settles how a body reaches another module.
[025][c] - Lowering writes one method per instantiation, test-first, named without collision.
[025][d] - A test on the bytecode: an `Int` passed to a generic is carried as `long` throughout.
[025][e] - Executable examples under `tests/spec/generics/`.

## 🔴 Item 026: A record that never escapes is never allocated
**Depends on:** Item 022, Item 023 — splitting a value into fields is legal only without identity.
A record built and read within one function has no reason to reach the heap.
Without identity, the compiler may keep such a value in locals, one per field, and never `new` it.
HotSpot's escape analysis does this when it can; Lumen's lowering does it every time it applies.
That is Valhalla's scalarization, available today, because the language meets its precondition.
An optimization earns its place by measurement, so the item starts with a number.
[026][a] - A measured baseline: a loop building a record per iteration, timed with the JDK.
[026][b] - `docs/specs/codegen.md` states the guarantee and exactly when a value stays in locals.
[026][c] - The lowering, test-first: such a record emits no `new`, asserted on the instructions.
[026][d] - A property test: the scalarized program computes what the allocating one computes.

## 🔴 Item 027: Every generated class keeps the shape of a value class
**Depends on:** Item 022, Item 023 — a value class has no identity, which those two make true.
Valhalla flattens a class that has no identity, no null, and equality by state.
Lumen's types have all three by design, so readiness is a matter of never drifting from it.
A value class is `final` with `final` fields, and declares nothing a value class may not.
The writer marks class and fields so, and a test holds every generated class inside the shape.
[027][a] - `docs/specs/codegen.md` states the shape every generated class keeps, and why.
[027][b] - Every generated field is `final` and every concrete class is `final`, held by a test.
[027][c] - A test that no generated method is `synchronized`, which a value class cannot lock on.

## 🔴 Item 028: Every generated class is a value class
**Depends on:** Item 027 — the flag is set only on a class already inside the shape.
JDK 28 ships value classes, and the project targets JDK 28 or later from now on.
The writer moves to class-file version 72 and marks every generated class as a value class.
If 28 still holds value classes in preview, the class file says so and `lumen run` enables it.
Nothing waits for a later JDK; readiness is the flag being set today, on an early-access build.
[028][a] - `docs/implementation.md` names JDK 28 as the floor, and settles the preview question.
[028][b] - The writer sets the flag, test-first against the bytes the class-file spec names.
[028][c] - `lumen run` passes what a preview class file needs, if 28 needs it, test-first.
[028][d] - A run-time example on a record, skipped with a named reason below JDK 28.

## 🔴 Item 029: No exceptions and no `unwrap`; every operation is total
**Depends on:** Item 007 — a total `/` needs the type of what it returns.
The non-goals name only checked exceptions; nothing says a Lumen program never throws or catches.
Today `/` and `%` lower to `LDIV` and `LREM`, so a zero divisor throws `ArithmeticException`.
The rule: no program can throw, catch, or observe an exception, and no operation is partial.
A partial function is written as `Option` or `Result` and taken apart only by `match` or `?`.
The prelude never gains `unwrap` or `expect`; there is no way to turn `None` into a crash.
The one throw the compiler emits, after an exhaustive `match`, stays unreachable by construction.
[029][a] - `docs/design.md` section 5 states the rule, and the non-goals name exceptions outright.
[029][b] - `docs/specs/arithmetic.md` settles what `/` and `%` do on a zero divisor, with no throw.
[029][c] - Lowering of `/` and `%` follows the spec, test-first; no generated method can throw.
[029][d] - `docs/specs/modules.md` states that the prelude takes `Option` apart only by `match`.
[029][e] - Executable examples under `tests/spec/arithmetic/`, including the zero-divisor case.
