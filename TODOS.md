# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

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
