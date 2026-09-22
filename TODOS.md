# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

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
