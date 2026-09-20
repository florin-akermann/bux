# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

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
