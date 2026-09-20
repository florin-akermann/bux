# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟢 Item 038: A `()` given to something that holds a reference
`Ok(())` is refused by the JVM verifier rather than by the compiler, and `Some(())` and `[()]` too.
`docs/specs/types.md` writes `Result<(), Error>` itself, so this is a type the language expects.
`docs/specs/codegen.md` says `()` is carried by nothing at all, and a field a type parameter left
open holds a reference, so lowering pushes nothing where a word is wanted and the bytes are short.
Lowering tells "no value, the type is `()`" from "no value, control left" by the same `None`, and
that conflation is the hole: the second is real, and the first needs something to push.
A record is already right, because a field typed `()` is not carried at all.
[038][a] - A failing test first, one per place a `()` reaches a reference: `Ok`, `Some`, a list.
[038][b] - Spec the answer in `docs/specs/codegen.md`, which today says no representation is read.
[038][c] - Lowering holds to it, and a JVM loads and verifies what comes out.
[038][d] - Executable examples that run, under `tests/spec/unit/`.
