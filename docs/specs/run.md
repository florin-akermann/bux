# Running a program

## Intent

`bux run <file> [argument...]` compiles a module and runs it.
It is the shortest way from a source file to the thing the source file does, and it is what an
executable example uses to prove a program does not only compile but works.

`docs/design.md` section 11 says where a program starts: `fn main(arguments: List<String>) -> Int`.
A module that declares `main` at that shape is a program; every other module is a library.

## What running amounts to

`bux run` does what `bux build` does and then hands the result to a JVM.
The class files are written as `bux build` writes them, so a run leaves what a build leaves.

The JVM is started on the module class.
Its class path is the one `target/` of the build, then each archive that the manifest states.
The next section states that directory, and the section after it states the archives.
The entry point is part of what is written and not something the runner supplies.
The runner adds `--enable-preview`, because every class written is a value class.
JDK 28 holds value classes in preview, and `docs/specs/codegen.md` says how the bytes look.
The runner also limits what the program can reach, and a section below states how.

The program's own output is the runner's output: what it writes to standard output and to
standard error, the runner passes through unchanged, and the status it ends with is the status
`bux run` ends with.

## Where a build writes

A build writes every class into one `target/`, in the directory of the module the command names.
A module of a package sits in the package's directory, so that is the package's `target/`.
That `target/` holds the module, each module it reaches, the library, and the prelude.
A module of another package is written there too, and not in the `target/` of that package.
So a build never writes into a directory of another package.
No class lands beside a source, and no option chooses another directory.
A clean build is `rm -rf target/`, because no other directory holds what a build writes.
`docs/specs/codegen.md` states the path of each class under `target/`.

A build also writes the resources of its compiler into `target/`, beside the classes.
The resources are the three directories the compiler reads: `library/`, `help/`, `explanations/`.
The build copies each directory from the first entry of its own class path that holds it.
It first deletes the old copy, so a resource that the compiler no longer holds is not left there.
It copies only the files that end in `.bx` or `.md`, which are the files the compiler reads.
So a build writes the resources that its compiler read.
A program that calls the compiler then finds the resources on its own class path.
An example is a test of `tests/` that calls `bux help` or `bux explain`.

That one `target/` is the whole class path, for a run and for the script below.
`bin/bux` starts `compiler/target/`, with that one directory on the class path and no other entry.

`bux test` is the one exception, and it writes nothing into `target/`.
It writes the classes and the resources of its run into a directory made for that run alone.
It then deletes that directory.
`docs/specs/doc-examples.md` states why.

## The archives on the class path

A `jar` line of the manifest names a Java archive, which `docs/specs/packages.md` states.
The class path of `bux run` is the one `target/`, then the archives.
The class path of `bux test` is the directory it writes its classes into, then the archives.
The archives are in the order of their lines, and nothing else is on the class path.
The archives of each dependency come after, in the order of the `depends` lines.
Each dependency puts its own archives first and then those of its dependencies.
A package that two routes reach puts its archives on the class path once, at the first place.

`target/` is first, so a class that the build writes is never hidden by a class of an archive.
A module with no manifest, and a package with no `jar` line, has `target/` alone.
The command line below writes the class path as `<dir>[:<archive>...]`.
No entry holds `:`, so `run`, and `test` with an archive, refuse a directory with one as `L0610`.

## What the program can reach

A program reaches its library, its stated archives, and the `java.base` module of the JDK.
It reaches nothing more.
The compiler refuses an `extern` that names a class outside, which `docs/specs/interop.md` states.
A class in a stated archive can still reach a class outside, so the JVM itself refuses those too.
The runner starts every compiled program with this command line, in this order:

```text
java --enable-preview --limit-modules java.base -Djdk.serialFilter=!* -cp <dir>[:<archive>...] <module> <word...>
```

`--limit-modules java.base` gives the JVM the `java.base` module and no other module.
A JVM without the flag has all the modules of the JDK, for example `java.naming`.
Each of `java.naming`, `java.rmi`, and `java.scripting` can load or run code from elsewhere.
With the flag, the JVM finds no such class, and a call of one fails with `NoClassDefFoundError`.
The library uses only `java.base`: `java.lang`, `java.io`, `java.nio.file`, and `java.util`.

`-Djdk.serialFilter=!*` refuses every class in a stream of serialized bytes.
Serialized bytes can make an object of any class that the JVM can load, and run its code.
A Bux value is never serialized, so the filter refuses nothing that a program needs.

The runner removes three variables from the environment of the program:

- `JAVA_TOOL_OPTIONS`, which the JVM reads.
- `JDK_JAVA_OPTIONS`, which the `java` launcher reads.
- `_JAVA_OPTIONS`, which the JVM of HotSpot reads.

Each variable adds options to the JVM, and an option such as `-javaagent` can change each class.
No other variable of the environment changes, so a program still reads its environment.
The JVM of the compiler still reads the three variables, because `bin/bux` starts it unchanged.

`run` and `test` build the command line with one function, `program_line`.
Both remove the variables with one function, `cleared`, in `compiler/command.bx`.
`tests/spec/interop/outside_java_base.bx` shows that the compiler refuses a class of `java.naming`.
`tests/started.bx` starts `run` and `test` with `JAVA_TOOL_OPTIONS` set.
It shows that the program sees no such variable and that its JVM writes nothing about one.

## The arguments

Every word after the file goes to the program, and nothing else does.

```sh
bux run report.bx --wide notes.txt
```

`--wide` and `notes.txt` reach the program, in that order, unchanged.
Nothing the runner writes is read as one of them, because they are written after the class the
JVM is started on.
The name of the program is not among them: a program already knows what it is.

The runner passes them to the JVM's own `main(String[])`, and the entry point the compiler writes
gathers that array into the `List<String>` it hands `main`.
A command that writes no word after the file runs the program with an empty list.

`--help` is one of those words, so `bux run` has no help flag of its own.
A word that meant one thing to the program and another to the runner would be a special case, and
the rule that every word after the file belongs to the program leaves no room for one.
`bux help run` is where this topic is read, and `bux --help` still names the command.

## The status

The run's status is the `Int` that `main` gives back.
A program that has nothing to say gives back `0`.

A status is eight bits wide on every system the JDK runs on.
So the entry point hands the JVM the low eight bits of the answer and nothing else, which maps
every `Int` onto one status and leaves the giving of a status total.
A program giving back `256` ends with `0`, and one giving back `-1` ends with `255`.

## Finding the JDK

A program is compiled before the JDK is looked for.
A program that does not compile is told so on a machine that could not have run it anyway, and a
module that is no program is told that before a JVM is reached for at all.

`JAVA_HOME` names the JDK, and nothing else is searched.
A search of `PATH` would run whichever JVM happened to be first, and a build that is reproducible
deserves a run that is too.

`JAVA_HOME` unset is refused, and so is a `JAVA_HOME` holding no `bin/java`.
Both name what was looked for, because the fix is to set one environment variable and the message
is where the reader learns which.

## The errors

None of these is a diagnostic: they are things about the run rather than about the program, so
they carry no code and point at no span.

```text
error: demo.bx: a module is run through `fn main(arguments: List<String>) -> Int`, which this one does not declare
error: JAVA_HOME is not set, and running a program needs the JDK it names
error: /opt/nothing/bin/java: JAVA_HOME names no JDK
```

A module declaring `main` with any other signature is the same as not declaring it, because
`fn main(arguments: List<String>) -> Int` is the one shape a program starts at.
The whole signature is read: the one parameter is `List<String>`, and the result is `Int`.
The message names that shape rather than the word alone, so a module declaring the wrong one is
told what to write instead of sent looking for what is already there.

## Exit codes

```text
1  the compiler refused the program
2  a file could not be read or written, the JDK was not found, or there is no `main`
```

Both are given before a program runs, so neither is ever a status a program chose.
Once a program runs, `bux run` ends with the status the program ended with, whatever it is.
A program the operating system stopped rather than let end has no status of its own, and is
reported as 128 plus the number of the signal.

## The command line written in Bux

`compiler/main.bx` is the command line written in Bux.
`compiler/command.bx` holds every command, because no module can import a module named `main`.
The help text is read as a class-path resource: a file in `compiler/help/`.

`bin/bux` is the launcher, a POSIX `sh` script.
It starts `java --enable-preview` on the class `main`, with every word it was given.
The class path is `compiler/target/` alone, which holds the classes and the resources.
A link to the launcher, as on `PATH`, works: the launcher follows the link to find `compiler/`.
It finds the JDK as this page says: `JAVA_HOME` names it, and nothing else is searched.
It refuses with status `2` when the compiler is not built, and when `JAVA_HOME` names no JDK.

```text
error: <root>/compiler/target/main.class: the compiler is not built; bin/bootstrap builds it
error: JAVA_HOME is not set, and the bux compiler runs on the JDK it names
error: /opt/nothing/bin/java: JAVA_HOME names no JDK
```

`run` starts the program with the streams of the command line.
The status of the command line is the status that the program ends with.
A program that a signal stops ends with the status the JVM gives it, which is 128 and the signal.

The JVM gives an error in words of its own, so the command line looks at the path instead.
A directory, a file that it cannot open, and a file that is not UTF-8 each get fixed words.
An example is `Is a directory (os error 21)`.
A path that ends in `/` and names a file is refused before it is read.
A file whose name does not end in `.bx` is refused as `L0605`, and its help names the new name.

`tests/commanded.bx` holds the command line to golden answers under `tests/commands/`.
`lines.txt` holds the argument parser, and `fixtures.txt` holds each command on each example.
A golden answer is the status and every line of each stream.
`tests/enacted.bx` holds the files that each command writes, and what each command says.
`tests/started.bx` holds each command that starts a JVM.
`tests/launched.bx` holds the launcher where it starts the compiler and where it cannot.
With no JDK, a test of `tests/` skips each check that starts a JVM and says why.

## The bootstrap

The Bux compiler builds itself from a seed, and `bin/bootstrap` does it.
The seed `bin/seed.jar` holds the classes of a Bux compiler that an earlier compiler built.
It is the one archive that the repository keeps, and `docs/implementation.md` section 6 says why.

`bin/bootstrap` is a POSIX `sh` script, and it needs only `JAVA_HOME` and the checkout.
It deletes `compiler/target/`, so no class of an old build is left.
It unpacks the seed into a temporary directory, with the `jar` tool of the JDK.
It then replaces the three directories of resources there with those of the checkout.
These are `library/`, `compiler/help/`, and `compiler/explanations/`.
The seed runs with that one directory as its class path.
The seed then builds stage 1, the classes in `compiler/target/`, which `bin/bux` starts.
So stage 1 holds the resources of the checkout, and not the old copies that the seed holds.
Stage 1 then builds stage 2 from a copy of `compiler/` in a temporary directory.
Stage 2 is in the `target/` of that copy.
That directory is outside the repository, and the script deletes it when it ends.

Stage 2 is stage 1 byte for byte.
Stage 2 has the same class files as stage 1, at the same paths, with the same bytes.
One `diff -rq` over the two `target/` directories compares the stages.
When they differ, the script names the first class of that answer and ends with status 1.
A class or a directory that only one stage holds is a difference too, and the script names it.
When `diff` cannot compare the stages, the script ends with the status of `diff`, which is 2.
A difference is a defect in the compiler under `compiler/`, and the fix goes there.
The script refuses with status `2` when `JAVA_HOME` names no JDK and when the seed is missing.

The script is the check of the bootstrap, because it compares the two stages itself.
`bin/bux test tests` runs on stage 1, so each test of `tests/` is a check of stage 1.

A build copies the resources of its compiler, not those of the checkout.
So `bin/bux build compiler/main.bx` copies the resources that `compiler/target/` holds already.
Run `bin/bootstrap` after a change to `library/`, `compiler/help/`, or `compiler/explanations/`.
The bootstrap is the one step that reads the resources of the checkout into stage 1.

## Properties

These hold, and each is checked:

1. A module that declares `main` at the one shape is written with an entry point a JVM starts
   at, and every other module is written without one.
2. Running a module leaves exactly the class files building it leaves.
   A build leaves no class beside a source: each class it writes is under `target/`.
3. A run ends with the low eight bits of the `Int` `main` gave back.
4. Every word after the file reaches the program unchanged, `--help` among them.
5. Stage 2 of the bootstrap is stage 1 byte for byte, and both stages pass `tests/spec`.
6. A checkout with no class file bootstraps with `JAVA_HOME` alone, and `bin/bux` then starts.
7. A program loads no class outside `java.base`, and its JVM reads none of the three variables.
8. A program reaches a class of each archive its manifest states, and of no other archive.
