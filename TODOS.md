# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 091: The compiler and `bin/runner` are fast because they run on Bux processes
Self-hosting is done, and `bin/runner` now runs so long that it looks stuck.
Section 15 of `docs/design.md` designs the concurrency, and this item builds it and uses it first.
The compiler and its checks are the first program that uses processes, so dogfood finds the gaps.
[091][a] - The wall time of `bin/bootstrap` and `bin/runner` is measured, and the slow part named.
[091][b] - `docs/specs/concurrency.md` answers the two open questions that section 15 leaves.
[091][c] - The spec states `process`, `spawn`, the handle, the bounded mailbox, and the deadline.
[091][d] - The Bux lexer, parser, and formatter accept `process` and `spawn`.
[091][e] - The type phase holds the one process shape, and each break of it gets a diagnostic.
[091][f] - `send` gives a typed result for a process that has ended, and a full mailbox waits.
[091][g] - The lowering runs each process on a JVM virtual thread, and no program can see how.
[091][h] - `tests/spec/concurrency/` holds a counter, a worker pool, and each refused shape.
[091][i] - `bin/runner` hands its checks to a pool of processes, one for each processor.
[091][j] - The compiler lowers and writes independent modules in processes of their own.
[091][k] - The new wall times are measured against the times of [091][a], and both are recorded.
[091][l] - Each gap or bottleneck that this dogfood finds becomes its own item in `TODOS.md`.
[091][m] - `bux help process` states the process shape, and section 15 drops its 0.4 note.

## 🔴 Item 080: `Option<()>` is refused wherever a program writes it or inference reaches it
`Some(())` says only that a value is there, which is `Bool` spelled a second way and nullability.
`Result<(), E>` stays, because its `Err` carries a reason; `docs/design.md` section 5 says why.
[080][a] - `docs/specs/types.md` states the refusal; `docs/specs/diagnostics.md` lists its code.
[080][b] - The Bux type phase refuses `Option<()>`, written or inferred.
[080][c] - `tests/spec/unit/carried.lm` carries `()` through `Result` only, next to a refused one.

## 🔴 Item 081: No Lumen name remains; the tree says Bux and every source is a `.bx` file
Lumen survives in 333 files, and 5665 sources still end in `.lm`, so each name has two spellings.
AGENTS.md makes a tree-wide rename one item that lands whole, and this item is that rename.
[081][a] - Every `.lm` source under `compiler/`, `library/`, `example/`, and `tests/` becomes `.bx`.
[081][b] - The compiler reads `.bx` sources only, and a `.lm` path gets a diagnostic, not a crash.
[081][c] - The JVM package `lumen/` in generated class files becomes `bux/`.
[081][d] - Docs, help text, hooks, `mycs.toml`, and comments say Bux; open todos say `.bx`.
[081][e] - The Name section in AGENTS.md drops the old name, and a search for `lumen` finds nothing.

## 🔴 Item 082: `let` makes a name that never changes and `var` a name that can; `:=` is gone
**Depends on:** Item 081 — this item lands on `.bx` files.
Today `x := 0` and `var x = 0` make a name with two operators, so a binding has two spellings.
`let x = 0` and `var x = 0` differ only in the keyword, and the keyword says what the name is.
A bare `=` then only changes a `var`, so each operator has one job, as in Swift.
[082][a] - `docs/design.md` binds with `let` in every example, and section 10 states the rule.
[082][b] - `docs/specs/grammar.md`, `lexer.md`, and `formatting.md` spell `let`; `:=` leaves them.
[082][c] - The Bux lexer, parser, and formatter accept `let` and drop the `:=` token.
[082][d] - A program that writes `x := 0` gets a diagnostic that says `write let x = 0`.
[082][e] - Every source under `compiler/`, `library/`, `example/`, and `tests/` binds with `let`.
[082][f] - A search for `:=` in sources, docs, and help text finds only the refusal test.

## 🔴 Item 083: A manifest names each Java archive by path and hash, and a run reaches no other
A package that reaches the Java ecosystem needs a way to say which archives it uses.
Nothing is fetched, so an archive is a file already on disk, and its hash pins its content.
[083][a] - `docs/specs/packages.md` states the manifest line, such as `jar lib/x.jar sha256:…`.
[083][b] - The build refuses an archive that is missing or whose hash is not the one stated.
[083][c] - The build refuses an archive whose `Class-Path` attribute names other archives.
[083][d] - `bux run` puts only the build directory and the stated archives on the class path.
[083][e] - `docs/implementation.md` says the archive line is for the JVM target only.

## 🔴 Item 084: An `extern` names only a class that the build can account for
**Depends on:** Item 083 — an accepted class is in a stated archive or on the allow list.
Today an `extern` can name `java.lang.Class.forName` and load a class whose name comes at run time.
The same form reaches `javax.naming.InitialContext.doLookup`, which is the Log4Shell call.
An accepted class is in a stated archive, or in a `java.base` package on a fixed allow list.
The allow list leaves out reflection, class loaders, method handles, and deserialization.
[084][a] - `docs/design.md` section 17 states the rule in the target-neutral words of Item 086.
[084][b] - `docs/specs/interop.md` states the JVM form, the allow list, and the new code.
[084][c] - The Bux phases read the entry names of each stated archive and refuse other classes.
[084][d] - A spec example shows an `extern` on `java.lang.Class.forName` that is refused.

## 🔴 Item 085: `bux run` starts a JVM that has only `java.base` and no injected options
The JVM starts with all its modules, so `java.naming`, `java.rmi`, and `java.scripting` load.
`JAVA_TOOL_OPTIONS`, `JDK_JAVA_OPTIONS`, and `_JAVA_OPTIONS` can add a `-javaagent` to a run.
A stated archive can still load a class at run time, and these flags narrow what it can reach.
[085][a] - `docs/specs/run.md` states the flags, the removed variables, and the reason for each.
[085][b] - The runner passes `--limit-modules java.base` and `-Djdk.serialFilter=!*`.
[085][c] - The runner removes the three variables from the environment of the JVM.
[085][d] - A spec example that `bin/runner` runs shows that no `javax.naming` class loads.
[085][e] - `bux help run` says a program reaches only its library and its stated archives.

## 🔴 Item 088: The class-file writer's copied defects are fixed in Bux
Item 073 copied seven defects of the Rust writer so that the bytes match; the spec names six.
[088][a] - A class name two modules write is refused, and no hierarchy entry is written twice.
[088][b] - A frame merge with locals or a stack of two lengths is a defect the writer reports.
[088][c] - A call and a constructor pop two stack slots for a `long`.
[088][d] - A branch to a label that never lands, or past the i16 range, is refused, not patched.
[088][e] - A Utf8 pool entry over 65535 bytes is refused, not capped.
[088][f] - The operand swap in the lowering adapts a unit operand, and the spec says so.

## 🔴 Item 089: A `test` block states a test, and `bux test` runs every test and example in a package
**Depends on:** Item 081 — `bux test` runs `.bx` files.
Today `lumen test` takes one file and runs only the `// example:` lines of that module.
One example line cannot hold a test that needs setup over several statements or needs `io`.
A `test "name" { … }` block at the top level of a module holds such a test, and its body is `Bool`.
A block is not a function, so it has no signature, no example, and no caller but the runner.
[089][a] - `docs/design.md` states the block and why an example line is not enough for it.
[089][b] - `docs/specs/testing.md` states the block, the run order, the report, and the exit codes.
[089][c] - The Bux lexer, parser, and formatter accept the block.
[089][d] - `bux build` and `bux run` leave every block out, so a test never ships in a program.
[089][e] - `bux test` with no argument runs the package in the current directory.
[089][f] - `bux test` on a directory runs every example and every block of every module in it.
[089][g] - The run reports every test that did not hold, with its module, its name, and its line.
[089][h] - `tests/spec/testing/` shows a block that holds, one that does not, and a refused one.
[089][i] - `bux help test` states the block and the package run, and `packages.md` agrees.

## 🔴 Item 090: A generic function called with its first argument in front builds a class the JVM refuses
Item 087 found it in both compilers, and the example that showed it was removed rather than kept.
`count.labelled("x")` with `fn labelled<T>(value: T, said: String) -> T` builds, and the run fails.
The JVM says `VerifyError: Operand stack underflow`: the lowering drops the argument with `pop2`.
[090][a] - `tests/spec/calls/` holds an `expect-run` example of such a call, on `T` and on `Int`.
[090][b] - The lowering in `compiler/` keeps the argument on the stack, and the example runs.
[090][c] - `compiler/exhaustiveness.lm`, `holes.lm`, and `types.lm` lose `printed`, which has no caller.
