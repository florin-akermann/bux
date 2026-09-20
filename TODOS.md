# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

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

## 🔴 Item 053: An `extern` reaches a member whose descriptor gives an `int`
`docs/specs/interop.md` carries `Int` as a `long`, and a JVM `int` is a type no Lumen type is.
`String.hashCode`, `String.length`, and `List.indexOf` each give one, so none of them is reachable.
`Hash<String>` therefore stays the compiler's own, which Item 048 left as the one supplied instance.
The compiler reads no class file, so it cannot learn a descriptor; the declaration has to say.
Whatever says it is a language change, so `docs/design.md` section 17 answers first.
[053][a] - `docs/design.md` section 17 says how a declaration names a member that gives an `int`.
[053][b] - `docs/specs/interop.md` states the widening and what it refuses, spec before code.
[053][c] - `Hash<String>` moves to `library/prelude.lm`, and nothing is supplied any more.
[053][d] - `strings.length` and `list.index_of` land, which `docs/specs/library.md` is waiting on.

## 🟡 Item 054: The specs still say the compiler supplies what the library writes
`library/prelude.lm` writes every instance of `Eq`, `Ord`, `Hash`, `Show`, and the operators.
`docs/specs/traits.md` still has a section called "What the compiler supplies" that lists them.
`docs/specs/operators.md` and `docs/specs/literals.md` say the same of what they are about.
Each was true before the prelude became Lumen source, and each is a claim no code holds up.
A reader who believes them looks in the compiler for a body that is in `library/prelude.lm`.
[054][a] - `docs/specs/traits.md` says which instances the library writes and which are supplied.
[054][b] - `docs/specs/operators.md` and `docs/specs/literals.md` take that same wording.
[054][c] - `crates/diagnostics/src/explanations/L0308.md` and `L0310.md` follow the specs.

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

## 🔴 Item 056: The IR's doc comments still say the compiler supplies an instance
Item 054 took the wording out of the specs, and the lowering's own comments still carry it.
`crates/ir/src/lower/operator.rs`, `literal.rs`, `expr.rs`, `derive.rs`, and `pattern.rs` each
say "a type the compiler supplies the instance for" where they mean one the JVM holds.
`crates/ir/src/lower/supplied.rs` opens by saying the prelude is not Lumen source yet, and it is.
The behaviour they describe is right: a use over `Bool`, `Int`, or `String` is the instruction.
What is wrong is where they send a reader looking for the body, which is `library/prelude.lm`.
[056][a] - The five files in `crates/ir/src/lower/` say which types those are and who writes them.
[056][b] - `crates/resolver/src/resolve/traits.rs` and `crates/ir/src/lower.rs` take that wording.
[056][c] - So do the tests that assert about it: `crates/types/tests/integration/traits.rs`, and
  `literals.rs`, `operators.rs`, and `traits.rs` under `crates/ir/tests/integration/`.
