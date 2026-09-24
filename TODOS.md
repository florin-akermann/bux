# TODOS

<!-- ID scheme: monotonically-increasing integers, zero-padded to 3 digits (#001, #002 …).
     Sub-tasks use the parent ID and a letter suffix ([001][a], [001][b] …).
     New items get the next available number. Never reuse an ID. -->

## Open

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
Today `bux test` takes one file and runs only the `// example:` lines of that module.
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
[103][c] - `tests/launched.bx` holds its 2 s window under a full pool, or states a wider one.

## 🔴 Item 104: `L0702` sees a clash of two class names that differ only in case
Item 099 found it: on a file system that ignores case, `List.class` and `list.class` are one file.
A build then writes one class over the other, and `L0702` compares the two names exactly.
The library no longer has such a pair, but a program's own types and modules can still meet.
The `L0702` message also names `bux` for a library class, because it reads up to the first `/`.
[104][a] - `docs/specs/codegen.md` states that two class names that differ only in case clash.
[104][b] - `L0702` refuses such a pair, and its message names the module that owns the class.
[104][c] - A `tests/spec/` example shows a program whose type and module differ only in case.

## 🔴 Item 105: A golden file holds every command on every example, and `fmt` changes no later answer
Item 092 found it: `tests/commanded.bx` runs every command line of a golden file in one stage.
So `$ fmt` rewrites the staged copy, and `$ build` of a refused file then records status 0.
`tests/spec/interop/outside_java_base.bx` has no entry at all, and nothing reports a missing one.
[105][a] - `docs/specs/executable-examples.md` states that each command sees the example as written.
[105][b] - `bin/runner` fails when an example under `tests/spec/` has no entry in `fixtures.txt`.
[105][c] - `bin/runner golden` adds the entries of a new example, and the missing entries are added.
