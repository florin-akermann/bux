# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 015: An executable example per public function
**Depends on:** Item 005, Item 010 — the harness runs the examples, and running them needs a JDK.
A public function carries at least one executable example, or the module does not compile.
The examples run with the test suite, so a signature never drifts from the evidence for it.
[015][a] - Spec first in `docs/specs/doc-examples.md`: the form, and the missing-example error.
[015][b] - Extraction from doc comments, test-first, with spans into the original file.
[015][c] - `lumen test` runs them; a failing example names the function it documents.
[015][d] - Executable examples: a public function without one fails to compile.


## 🟡 Item 017: A parameter is never a bare `Bool`
**Depends on:** Item 016 — the rule shares its spec and its error voice with named arguments.
mycs lints `Flag Argument`; the language removes it by refusing a `Bool` parameter outright.
A two-variant ADT takes its place, so `open(ReadOnly)` reads where `open(true)` did not.
The project already prefers one sum type over a pair of flags; this makes it the only option.
[017][a] - `docs/design.md` section 11 states the rule; `docs/specs/arguments.md` gives the error.
[017][b] - The spec settles the one carve-out: a boolean operation over `Bool` is still writable.
[017][c] - The check after inference, test-first; the `help:` names the two-variant ADT to write.
[017][d] - Executable examples under `tests/spec/arguments/`, including the compile-fail case.

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

## 🔴 Item 032: A record field of record type is laid out flat
**Depends on:** Item 026 — the same measurement, on a record that holds a record.
`docs/design.md` section 1 promises a record laid out flat wherever the JVM can flatten one.
A JVM decides a field's layout when it loads the class, before the field's own class is loaded.
JEP 401's `LoadableDescriptors` attribute names the classes a class file wants loaded first.
Without it a `User` holding an `Address` keeps a reference where the JVM would have flattened.
An optimization earns its place by measurement, so the item starts with a number.
[032][a] - A measured baseline: a loop reading a record held in a record, timed with the JDK.
[032][b] - `docs/specs/codegen.md` states which descriptors the attribute names, and on which class.
[032][c] - The writer emits the attribute, test-first; a test on the bytes asserts what it names.
[032][d] - The measurement again, and the number beside the baseline in the item's commit.

## 🔴 Item 031: An executable example asserts what a program writes out
**Depends on:** Item 029 — no Lumen program can fail at runtime, so an exit status proves little.
`// expect-run` judges an example by its exit status, which is `0` for every program that runs.
Now that no operation throws, an example cannot show that a value is the one the spec claims.
An example gains a way to state the output it must produce, and the harness compares it.
`17 / 5` being `Some(3)` is then checked rather than asserted in prose.
`io.println` is named in a help line and implemented nowhere, so writing out comes first.
[031][a] - A program has a way to write a line out, which version 0.1 has nowhere yet.
[031][b] - `docs/specs/executable-examples.md` states the header and what is compared.
[031][c] - The harness compares the output, test-first; a mismatch names the file and both texts.
[031][d] - `tests/spec/arithmetic/division.lm` writes its answers out and states them.

