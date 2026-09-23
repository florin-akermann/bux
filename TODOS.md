# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

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

## 🔴 Item 092: Imports form one block with no blank line between two of them
Today canonical form puts one blank line between two top-level items, and an import is one.
A file with ten imports thus spends ten lines on blanks, and the block does not read as one.
The new rule: no blank line between two imports, and one blank line after the last import.
`bux fmt` repairs the whitespace, and a file with a blank line between imports does not compile.
[092][a] - `docs/specs/formatting.md` and `docs/design.md` section 13 state the rule.
[092][b] - The printer in `compiler/format.lm` writes imports with no blank line between them.
[092][c] - A blank line between two imports is `L0200`, and its `help:` says to run `bux fmt`.
[092][d] - The examples in `compiler/format.lm` that show two imports use the new form.
[092][e] - `tests/spec/` holds a canonical import block and a refused one with a blank line.
[092][f] - Every source under `compiler/`, `library/`, `example/`, and `tests/` is formatted again.
[092][g] - `bux help fmt` states the rule, and `bin/runner` shows the whole tree is canonical.

## 🔴 Item 096: A build writes every class file under `target/`, and no class lands beside a source
Today a build writes each class beside its source, so `compiler/` and `tests/` hold about 2000.
A class file among the sources hides the source tree, and each script must find and delete them.
The new rule: every class lands in `target/`, on the same level as the sources and `tests/`.
In `target/`, a class keeps the path its module gives it: `compiler/ir.lm` writes `target/ir/`.
A clean build is one step: delete `target/`, and no other directory holds compiler output.
[096][a] - `docs/specs/codegen.md`, `docs/specs/run.md`, and `docs/implementation.md` state it.
[096][b] - `compiler/command.lm` writes each class under `target/`, and never beside the source.
[096][c] - `bux run` and `bux test` put `target/` on the class path, and every example still runs.
[096][d] - `bin/bootstrap`, `bin/bux`, and `bin/runner` read and delete classes only in `target/`.
[096][e] - `.gitignore` names `target/`, and the `*.class` line and its comment say why.
[096][f] - A `tests/spec/` example shows that a build leaves no class beside its source.
[096][g] - `bux help build` states where the classes go.

## 🔴 Item 099: A module named as a library module runs, because its class is not the library's
Item 094 found it: `bux test tests/spec/packages/carried/strings.lm` ends with `NoSuchMethodError`.
The run writes a class `strings` beside the module, and the class path holds the library's `strings`.
So a call of `strings.length` reaches the wrong class, and `bin/runner` skips that module today.
`docs/specs/packages.md` says such a module is reached as the one the command names, so it must run.
[099][a] - `docs/specs/codegen.md` states the class name of a library module, distinct from a program's.
[099][b] - `bux run` and `bux test` on that module work, and the skip in `tests/documented.lm` goes.
[099][c] - A `tests/spec/` example runs a module named as a library module and calls that library.

## 🔴 Item 100: `spawn` starts a process that another module declares
Item 091 found it: `L0800` refuses a `spawn` of a process outside the module that declares it.
A library process, such as a ticker or a bus, is then out of reach, and section 15 wants both.
[100][a] - `docs/specs/concurrency.md` states how a process is reached through a module name.
[100][b] - The resolver reaches a process of another module, and `L0800` keeps its other refusals.
[100][c] - A `tests/spec/concurrency/` example spawns a process that another module declares.

## 🔴 Item 101: A foreign reference given to a process is refused
Item 091 found it: `docs/design.md` section 14 refuses a foreign reference given to a process.
No check holds that rule today, so a `spawn` argument or a message can carry one.
[101][a] - `docs/specs/concurrency.md` states the refusal and its code.
[101][b] - The type phase refuses a `spawn` argument and a message that hold a foreign reference.
[101][c] - A `tests/spec/concurrency/` example shows the refusal.

## 🔴 Item 102: A process that a platform error stops has a stated end, and `Delivered` is exact
Item 091 found it: a JVM error, such as a stack overflow, stops a process and is printed.
`ended` then stops its caller with the same error, so one stopped worker stops `bin/runner`.
Section 15 says no process fails, so the spec must say what such an end is and who learns of it.
A message can also be `Delivered` after `Done`, because the check and the offer are two steps.
[102][a] - `docs/specs/concurrency.md` states the end, and how `ended` and `send` report it.
[102][b] - A `tests/spec/concurrency/` example shows a process that a platform error stops.
[102][c] - `Delivered` means the mailbox took the message before the end, and a property shows it.

## 🔴 Item 103: `bin/runner` prints a part as it ends, and example runs share written classes
Item 091 found it: the pool prints every part line after `ended`, so the run shows nothing for 30 s.
The example-line jobs of `compiler/` and `tests/` write the same classes again, and each takes 27 s.
[103][a] - The runner prints each part line when that part ends, and the line order is stable.
[103][b] - The example-line runs of one module share one set of written classes.
[103][c] - `tests/launched.lm` holds its 2 s window under a full pool, or states a wider one.
