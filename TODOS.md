# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🟢 Item 052: A foreign reference is not sendable
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

## 🔴 Item 055: An `extern` reaches a member of a Java interface
`extern method` is lowered to `invokevirtual`, which the JVM refuses to link on an interface.
`extern type Path = "java.nio.file.Path"` with `extern method as_text(path: Path) -> String`
compiles, and running it is `java.lang.IncompatibleClassChangeError` with no diagnostic before it.
Which of the two a Java name is is written in that name's own class file, and the compiler reads
none of them, so the author is the one who can say which it is.
[055][a] - `docs/design.md` section 17 says an `extern type` states that the class is an interface.
[055][b] - `docs/specs/interop.md` and `docs/specs/grammar.md` take it, and drop the paragraph
  that says reaching an interface waits.
[055][c] - The lowering emits `InvokeInterface` on such a receiver, with the constant pool entry
  an interface method reference is, and `tests/spec/interop/` runs one on a JDK.
