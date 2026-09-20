# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟢 Item 025: Generics are specialized, never erased
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

## 🔴 Item 037: A list is written, not only walked
**Depends on:** Item 025 — a list holds a type parameter, and a generic is specialized first.
`docs/specs/types.md` gives `List<T>` a `for` that walks one and no syntax that builds one.
So a function over a list is one nothing can call, and `for … in` is a loop no module can run.
`docs/specs/doc-examples.md` then keeps `// example: true` for functions nothing can state.
A way to write a list removes all three at once, and that carve-out goes with it.
[037][a] - Spec first in `docs/specs/types.md`: what writes a list, and what it costs to build one.
[037][b] - The grammar and inference take it, test-first; the element type comes from the writing.
[037][c] - Lowering writes it, and a JVM loads and verifies what comes out.
[037][d] - Executable examples under `tests/spec/lists/`: a list written, walked, and given away.
[037][e] - `// example: true` goes from `crates/cli/tests/integration/`, and from the specs.
