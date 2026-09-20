# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟡 Item 048: A Java class is reached through an `extern` declaration
**Depends on:** Item 044, Item 045 — a wrapper is a library module offering its own types.
`docs/implementation.md` section 3 sketches `extern java class` and says nothing of the boundary.
`docs/design.md` section 2 says what never crosses: identity, `null`, exceptions, the object model.
An `extern` declaration names a static method or a constructor and gives it a Lumen signature.
A `null` given back becomes `None`, a thrown exception becomes `Err`, and nothing else shows.
A Java object a Lumen program holds is a value it cannot compare, hash, or print until a trait says.
`io` and `files` become Lumen modules over `extern` declarations; the compiler stops supplying them.
[048][a] - `docs/design.md` gains a section: what an `extern` reaches, and what never crosses.
[048][b] - Spec first in `docs/specs/interop.md`: the declaration, the two mappings, and the errors.
[048][c] - Lowering emits the call and the two mappings, test-first, asserted on the instructions.
[048][d] - `io` and `files` in Lumen; `docs/specs/io.md` loses the "supplied module" paragraph.
[048][e] - Executable examples under `tests/spec/interop/`, skipped by name when there is no JDK.
[048][f] - `Show<Bool|Int|String>` and `Hash<String>` move to `library/prelude.lm` over `extern`.

## 🔴 Item 051: Identity is quarantined, not abolished
`docs/design.md` section 15 argues that no two spawned functions ever hold the same value.
The section's own example refutes it: `events` is held by the parent and by the spawned `produce`.
A channel is identity-bearing, because two holders of one channel is the whole point of a channel.
`send` mutates a queue and `receive` observes the mutation, which is what makes the example work.
The rule that survives the example is narrower and stronger than the one the section states.
A channel and a scoped resource are the only identity-bearing things, and neither is declarable.
Every type a program declares is a value, so a program has nothing to build a lock out of.
A channel is written by the compiler and the runtime, which is why it is the one safe exception.
[051][a] - `docs/design.md` section 15 states the narrower rule and drops the overreaching one.
[051][b] - Sections 10 and 14 take that wording, so the three say one thing about identity.
[051][c] - `docs/principles.md` line 10 gains the scope it assumes: a declared type has no identity.

## 🔴 Item 052: A foreign reference is not sendable
`docs/design.md` section 14 refuses a resource-typed value returned, stored in a field, or sent.
That "or sent" clause is the whole of the concurrency rule, and section 15 leans on it unsaid.
A Java object has identity and mutation, so one sent on a channel is shared mutable state again.
Every clause section 15 argues from fails the moment a foreign reference crosses a spawn.
The answer is the rule already written: a foreign reference escapes exactly as a resource does.
That is no new mechanism, and it is why the escape check is one rule rather than a family of them.
Item 048 lands the declaration that makes such a reference writable, so this item comes before it.
[052][a] - `docs/design.md` section 14 names a foreign reference among what the check refuses.
[052][b] - Section 15 cites the clause where it argues nothing is shared, so the two sections agree.
[052][c] - Item 048's `docs/specs/interop.md` points at the clause rather than restating the rule.
