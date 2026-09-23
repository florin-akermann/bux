# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

## 🔴 Item 080: `Option<()>` is refused wherever a program writes it or inference reaches it
**Depends on:** Item 087 — after the deletion, only the Bux type phase has to change.
`Some(())` says only that a value is there, which is `Bool` spelled a second way and nullability.
`Result<(), E>` stays, because its `Err` carries a reason; `docs/design.md` section 5 says why.
[080][a] - `docs/specs/types.md` states the refusal; `docs/specs/diagnostics.md` lists its code.
[080][b] - The Bux type phase refuses `Option<()>`, written or inferred.
[080][c] - `tests/spec/unit/carried.lm` carries `()` through `Result` only, next to a refused one.

## 🔴 Item 081: No Lumen name remains; the tree says Bux and every source is a `.bx` file
Lumen survives in 333 files, and 5665 sources still end in `.lm`, so each name has two spellings.
**Depends on:** Item 087 — the deletion removes the `lumen-*` crates, so none is renamed.
AGENTS.md makes a tree-wide rename one item that lands whole, and this item is that rename.
[081][a] - Every `.lm` source under `compiler/`, `library/`, `example/`, and `tests/` becomes `.bx`.
[081][b] - The compiler reads `.bx` sources only, and a `.lm` path gets a diagnostic, not a crash.
[081][c] - The JVM package `lumen/` in generated class files becomes `bux/`.
[081][d] - Docs, help text, hooks, `mycs.toml`, and comments say Bux; open todos say `.bx`.
[081][e] - The Name section in AGENTS.md drops the old name, and a search for `lumen` finds nothing.

## 🔴 Item 082: `let` makes a name that never changes and `var` a name that can; `:=` is gone
**Depends on:** Item 081, Item 087 — this item lands on `.bx` files, in the Bux compiler only.
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
**Depends on:** Item 087 — the build and `bux run` change in the Bux compiler only.
A package that reaches the Java ecosystem needs a way to say which archives it uses.
Nothing is fetched, so an archive is a file already on disk, and its hash pins its content.
[083][a] - `docs/specs/packages.md` states the manifest line, such as `jar lib/x.jar sha256:…`.
[083][b] - The build refuses an archive that is missing or whose hash is not the one stated.
[083][c] - The build refuses an archive whose `Class-Path` attribute names other archives.
[083][d] - `bux run` puts only the build directory and the stated archives on the class path.
[083][e] - `docs/implementation.md` says the archive line is for the JVM target only.

## 🔴 Item 084: An `extern` names only a class that the build can account for
**Depends on:** Item 083, Item 087 — stated archives, and the Bux compiler only.
Today an `extern` can name `java.lang.Class.forName` and load a class whose name comes at run time.
The same form reaches `javax.naming.InitialContext.doLookup`, which is the Log4Shell call.
An accepted class is in a stated archive, or in a `java.base` package on a fixed allow list.
The allow list leaves out reflection, class loaders, method handles, and deserialization.
[084][a] - `docs/design.md` section 17 states the rule in the target-neutral words of Item 086.
[084][b] - `docs/specs/interop.md` states the JVM form, the allow list, and the new code.
[084][c] - The Bux phases read the entry names of each stated archive and refuse other classes.
[084][d] - A spec example shows an `extern` on `java.lang.Class.forName` that is refused.

## 🔴 Item 085: `bux run` starts a JVM that has only `java.base` and no injected options
**Depends on:** Item 087 — the runner is the one in `compiler/command.lm`, and its test is Bux.
The JVM starts with all its modules, so `java.naming`, `java.rmi`, and `java.scripting` load.
`JAVA_TOOL_OPTIONS`, `JDK_JAVA_OPTIONS`, and `_JAVA_OPTIONS` can add a `-javaagent` to a run.
A stated archive can still load a class at run time, and these flags narrow what it can reach.
[085][a] - `docs/specs/run.md` states the flags, the removed variables, and the reason for each.
[085][b] - The runner passes `--limit-modules java.base` and `-Djdk.serialFilter=!*`.
[085][c] - The runner removes the three variables from the environment of the JVM.
[085][d] - A spec example that the Item 087 runner runs shows that no `javax.naming` class loads.
[085][e] - `bux help run` says a program reaches only its library and its stated archives.

## 🔴 Item 087: The Rust crates are deleted, and a `bux`-driven runner holds `tests/spec`
Item 075 landed stage 2 equal to stage 1 byte for byte, and every harness is a Rust test binary.
Until the deletion, each language change goes into two compilers that must agree.
So the deletion comes before the open language items, and each of them then changes Bux only.
No class file is kept today, so a checkout needs the Rust compiler to build stage 1.
[087][a] - A `bux`-driven runner runs every example under `tests/spec` and every `// example:`.
[087][b] - Every hegeltest property a Bux phase has is written in Bux, or its loss is stated.
[087][c] - The Rust crates and the Cargo workspace are deleted; the commit hook runs the runner.
[087][d] - `docs/implementation.md` section 6 says what remains, and AGENTS.md names no crate.
[087][e] - A stage-1 seed archive is kept in the repository, and `.gitignore` keeps it.
[087][f] - A fresh checkout builds the compiler from the seed with a JDK only, and a test shows it.
[087][g] - `docs/implementation.md` says when the seed is replaced: a source it cannot compile.
[087][h] - `README.md` states the status after Item 075 and builds with the seed, with no Cargo.

## 🔴 Item 088: The class-file writer's copied defects are fixed in Bux
**Depends on:** Item 087 — while Rust ships, the Bux writer must match it byte for byte.
Item 073 copied seven defects of the Rust writer so that the bytes match; the spec names six.
[088][a] - A class name two modules write is refused, and no hierarchy entry is written twice.
[088][b] - A frame merge with locals or a stack of two lengths is a defect the writer reports.
[088][c] - A call and a constructor pop two stack slots for a `long`.
[088][d] - A branch to a label that never lands, or past the i16 range, is refused, not patched.
[088][e] - A Utf8 pool entry over 65535 bytes is refused, not capped.
[088][f] - The operand swap in the lowering adapts a unit operand, and the spec says so.

## 🔴 Item 089: A `test` block states a test, and `bux test` runs every test and example in a package
**Depends on:** Item 081, Item 087 — `bux test` runs `.bx` files, in the Bux compiler only.
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
