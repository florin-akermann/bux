# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 073: Lowering and the class-file writer are written in Bux
**Depends on:** Item 061, Item 071 — the writer puts bytes on disk, and lowering reads types.
The writer has 172 sites of narrow integers, and a `bytes` module hides `% 256` behind names.
[073][a] - `compiler/ir.bx` lowers the typed tree to the JVM IR `docs/specs/codegen.md` states.
[073][b] - `compiler/jvm.bx` writes a class file, with a `bytes` module for `u1`, `u2`, and `u4`.
[073][c] - The harness holds every class file byte for byte equal to the Rust writer's output.

## 🔴 Item 074: The `bux` command line is written in Bux
**Depends on:** Item 060, Item 062, Item 068, Item 072, Item 073 — every command is a phase.
[074][a] - `compiler/main.bx` parses the arguments and runs every command `bux --help` lists.
[074][b] - A `bux` launcher script starts the JVM with `--enable-preview` on the compiled compiler.
[074][c] - Every executable example under `tests/spec` passes under the launcher.

## 🔴 Item 075: The compiler compiles itself
**Depends on:** Item 074 — the fixpoint needs the whole compiler.
[075][a] - Stage 1, built by the Rust `bux`, builds stage 2 from the same source.
[075][b] - A harness holds stage 2 equal to stage 1 byte for byte.
[075][c] - The Rust crates are deleted, and `docs/implementation.md` section 6 says what remains.

## 🔴 Item 080: `Option<()>` is refused wherever a program writes it or inference reaches it
**Depends on:** Item 071 — the Rust and the Bux type phases refuse it alike, at the same span.
`Some(())` says only that a value is there, which is `Bool` spelled a second way and nullability.
`Result<(), E>` stays, because its `Err` carries a reason; `docs/design.md` section 5 says why.
[080][a] - `docs/specs/types.md` states the refusal; `docs/specs/diagnostics.md` lists its code.
[080][b] - The Rust and the Bux type phases refuse `Option<()>`, written or inferred, alike.
[080][c] - `tests/spec/unit/carried.lm` carries `()` through `Result` only, next to a refused one.

## 🔴 Item 081: No Lumen name remains; the tree says Bux and every source is a `.bx` file
Lumen survives in 333 files, and 5665 sources still end in `.lm`, so each name has two spellings.
AGENTS.md makes a tree-wide rename one item that lands whole, and this item is that rename.
[081][a] - Every `.lm` source under `compiler/`, `library/`, `example/`, and `tests/` becomes `.bx`.
[081][b] - The compiler reads `.bx` sources only, and a `.lm` path gets a diagnostic, not a crash.
[081][c] - Every `lumen-*` crate becomes `bux-*`, and the `lumen` binary becomes `bux`.
[081][d] - The JVM package `lumen/` in generated class files becomes `bux/`.
[081][e] - Docs, help text, hooks, `mycs.toml`, and comments say Bux; open todos say `.bx`.
[081][f] - The Name section in AGENTS.md drops the old name, and a search for `lumen` finds nothing.

## 🔴 Item 082: `let` makes a name that never changes and `var` a name that can; `:=` is gone
**Depends on:** Item 081 — both items edit every source, so this one lands on `.bx` files.
Today `x := 0` and `var x = 0` make a name with two operators, so a binding has two spellings.
`let x = 0` and `var x = 0` differ only in the keyword, and the keyword says what the name is.
A bare `=` then only changes a `var`, so each operator has one job, as in Swift.
[082][a] - `docs/design.md` binds with `let` in every example, and section 10 states the rule.
[082][b] - `docs/specs/grammar.md`, `lexer.md`, and `formatting.md` spell `let`; `:=` leaves them.
[082][c] - The Rust lexer, parser, and formatter accept `let` and drop the `:=` token.
[082][d] - A program that writes `x := 0` gets a diagnostic that says `write let x = 0`.
[082][e] - The Bux lexer, parser, and formatter under `compiler/` change the same way.
[082][f] - Every source under `compiler/`, `library/`, `example/`, and `tests/` binds with `let`.
[082][g] - A search for `:=` in sources, docs, and help text finds only the refusal test.

## 🔴 Item 083: Only a library module writes `extern`, so no program loads a Java class by name
Today any module can write `extern static` on `java.lang.Class.forName` and load a class by name.
The same form reaches `javax.naming.InitialContext.doLookup`, which is the Log4Shell call.
`docs/design.md` section 17 says a program reaches Java through the library, but nothing checks it.
[083][a] - `docs/design.md` section 17 and `docs/specs/interop.md` state the rule and its code.
[083][b] - The Rust phases refuse an `extern` outside `library/`, with a code and an explanation.
[083][c] - The Bux phases under `compiler/` refuse it alike, at the same span.
[083][d] - The two `compiler/` modules that write `extern` reach Java through a library module.
[083][e] - The fixtures in `tests/spec/interop/` and `tests/spec/parser/` keep the boundary tested.
[083][f] - A spec example shows a program that writes `extern` and is refused.

## 🔴 Item 084: Every Java class the library names is on one fixed allow list
**Depends on:** Item 083 — the library is then the only place an `extern` can name a class.
An allow list makes a new `extern` on `ClassLoader`, reflection, or `javax.naming` fail a test.
A deny list is not enough, because it cannot name a class that nobody thought of.
[084][a] - `docs/specs/library.md` states the rule and names the list.
[084][b] - A test reads every `extern` in `library/` and fails on a class not on the list.
[084][c] - The list holds only the `java.base` classes that the library names today.

## 🔴 Item 085: `lumen run` starts a JVM that has only `java.base` and no injected options
The JVM starts with all its modules, so `java.naming`, `java.rmi`, and `java.scripting` load.
`JAVA_TOOL_OPTIONS`, `JDK_JAVA_OPTIONS`, and `_JAVA_OPTIONS` can add a `-javaagent` to a run.
[085][a] - `docs/specs/run.md` states the flags, the removed variables, and the reason for each.
[085][b] - The runner passes `--limit-modules java.base` and `-Djdk.serialFilter=!*`.
[085][c] - The runner removes the three variables from the environment of the JVM.
[085][d] - A run-time example shows that `Class.forName` finds no `javax.naming` class.
[085][e] - `lumen help run` says which modules a program has.
