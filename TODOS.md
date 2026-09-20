# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 033: Every operator is a trait method
**Depends on:** typeclasses, `docs/implementation.md` section 10 — an instance needs a trait.
`docs/design.md` section 8 makes `+ - * / %`, prefix `-`, and the comparisons trait methods.
The type checker wires them to `Int` and `String` by name today, so no declared type can own one.
A declared `Int32` gets `a + b` by writing an `Add` instance, as `Int` gets it from the library.
`==` already works this way, and the other operators follow it.
`&&`, `||`, and `!` stay `Bool`'s alone, because they short-circuit and an instance cannot.
[033][a] - Spec first in `docs/specs/operators.md`: each trait, its method, and the `Int` instances.
[033][b] - The library ships the `Int` and `String` instances; the checker resolves through them.
[033][c] - The hard-wired `Int` cases in inference are removed, test-first; `Int` tests still pass.
[033][d] - Executable examples under `tests/spec/operators/`: a declared type owning `+`, `<`, `/`.

## 🔴 Item 034: A literal takes the type its context expects
**Depends on:** Item 033 — a literal is a trait method, and the operator traits land first.
`1` is an `Int` today, at the one place inference types a literal, so `Int32` is written `Int32(1)`.
`docs/design.md` section 8 gives a whole-number literal the type its context expects instead.
That needs the type to have `IntegerLiteral`; a literal that nothing settles is still an `Int`.
A literal that does not fit its type is a compile error where it is written, never a wrapped value.
`let x: Int32 = 5_000_000_000` is refused; how an instance states what fits is the spec's question.
[034][a] - Spec first in `docs/specs/literals.md`: the trait, its bounds, and the error text.
[034][b] - Inference gives a literal a variable constrained by `IntegerLiteral`, defaulted to `Int`.
[034][c] - The fit check at compile time, test-first; the `Int` instance accepts every literal.
[034][d] - Executable examples under `tests/spec/literals/`: `Int32` as a literal, and a misfit.

## 🟡 Item 039: A module is loaded from a file
**Depends on:** nothing; version 0.1 lists basic modules in `docs/implementation.md` section 9.
Nothing in the toolchain reads a second file yet, which `docs/specs/modules.md` states.
`io` and `files` are supplied by the compiler, and the prelude is a list in the resolver.
Both become Lumen source once a module can be loaded, and 0.1 is not done until it can.
`docs/design.md` section 16 says one file is one module, named by its file.
Where an import finds its file, and what a missing one says, is what the spec must answer.
[039][a] - Spec first in `docs/specs/modules.md`: where an import finds its file, and the errors.
[039][b] - The toolchain reads every file a program imports, test-first; a cycle is refused.
[039][c] - A name reached through an import resolves and types against what that file declares.
[039][d] - `lumen build` writes the class files of every module the program reaches.
[039][e] - Executable examples under `tests/spec/modules/`: two files, and a missing import.

## 🔴 Item 040: `?` propagates an `Option`
**Depends on:** nothing; `docs/design.md` section 5 and `docs/specs/arithmetic.md` state the rule.
`/` and `%` give an `Option<Int>`, and today `?` is `Result`'s alone, so `(a / b)?` is refused.
Arithmetic that divides is then a `match` or an `or` per division, and stops reading as arithmetic.
`?` on an `Option` in a function that gives back an `Option` gives the value, or the `None` back.
`((total / count)? / 2)? + 2 * 5` is then the readable form, and each `?` marks a division.
A `None` met where the function gives back a `Result` stays `L0400`, because it names no error.
Nothing converts: `?` propagates each kind into a function that gives back the same kind.
[040][a] - Inference takes `?` on an `Option` in an `Option` function, test-first; `Result` as before.
[040][b] - Lowering hands the held `None` back as the answer; a property covers the shape it emits.
[040][c] - `tests/spec/arithmetic/division.lm` divides through `?`; a `Result` function is refused.
