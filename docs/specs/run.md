# Running a program

## Intent

`lumen run <file> [argument...]` compiles a module and runs it.
It is the shortest way from a source file to the thing the source file does, and it is what an
executable example uses to prove a program does not only compile but works.

`docs/design.md` section 11 says where a program starts: `fn main(arguments: List<String>) -> Int`.
A module that declares `main` at that shape is a program; every other module is a library.

## What running amounts to

`lumen run` does what `lumen build` does and then hands the result to a JVM.
The class files are written beside the source, exactly as `lumen build` writes them, so a run
leaves the same artefacts a build does and nothing more.

The JVM is started on the module class, in the directory the classes were written to.
Running the module class directly with `java --enable-preview` does the same thing, because the
entry point is part of what is written and not something the runner supplies.
The flag is the one thing the runner adds: every class written is a value class, which JDK 28
holds in preview, and `docs/specs/codegen.md` says what that makes the bytes look like.

The program's own output is the runner's output: what it writes to standard output and to
standard error, the runner passes through unchanged, and the status it ends with is the status
`lumen run` ends with.

## The arguments

Every word after the file goes to the program, and nothing else does.

```sh
lumen run report.lm --wide notes.txt
```

`--wide` and `notes.txt` reach the program, in that order, unchanged.
Nothing the runner writes is read as one of them, because they are written after the class the
JVM is started on.
The name of the program is not among them: a program already knows what it is.

The runner passes them to the JVM's own `main(String[])`, and the entry point the compiler writes
gathers that array into the `List<String>` it hands `main`.
A command that writes no word after the file runs the program with an empty list.

`--help` is one of those words, so `lumen run` has no help flag of its own.
A word that meant one thing to the program and another to the runner would be a special case, and
the rule that every word after the file belongs to the program leaves no room for one.
`lumen help run` is where this topic is read, and `lumen --help` still names the command.

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
error: demo.lm: a module is run through `fn main(arguments: List<String>) -> Int`, which this one does not declare
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
Once a program runs, `lumen run` ends with the status the program ended with, whatever it is.
A program the operating system stopped rather than let end has no status of its own, and is
reported as 128 plus the number of the signal.

## The command line written in Bux

`compiler/main.lm` is the command line written in Bux.
`compiler/command.lm` holds every command, because no module can import a module named `main`.
The help text is read as a class-path resource: a file in `compiler/help/`.

`bin/bux` is the launcher, a POSIX `sh` script.
It starts `java --enable-preview` on the class `main`, with every word it was given.
The class path is `compiler/`, with the help text, and the directory above it, with the library.
A link to the launcher, as on `PATH`, works: the launcher follows the link to find `compiler/`.
It finds the JDK as this page says: `JAVA_HOME` names it, and nothing else is searched.
It refuses with status `2` when the compiler is not built, and when `JAVA_HOME` names no JDK.

```text
error: <root>/compiler/main.class: the compiler is not built; bin/bootstrap builds it
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

`tests/commanded.lm` holds the command line to golden answers under `tests/commands/`.
`lines.txt` holds the argument parser, and `fixtures.txt` holds each command on each example.
A golden answer is the status and every line of each stream.
`tests/enacted.lm` holds the files that each command writes, and what each command says.
`tests/started.lm` holds each command that starts a JVM.
`tests/launched.lm` holds the launcher where it starts the compiler and where it cannot.
With no JDK, the runner skips each check that starts a JVM and says why.

## The bootstrap

The Bux compiler builds itself from a seed, and `bin/bootstrap` does it.
The seed `bin/seed.jar` holds the classes of a Bux compiler that an earlier compiler built.
It is the one archive that the repository keeps, and `docs/implementation.md` section 6 says why.

`bin/bootstrap` is a POSIX `sh` script, and it needs only `JAVA_HOME` and the checkout.
It deletes every class file under `compiler/`, so no class of an old build is left.
The seed then builds stage 1, the classes beside the sources in `compiler/`, which `bin/bux` starts.
Stage 1 then builds stage 2 from a copy of `compiler/` in a temporary directory.
That directory is outside the repository, and the script deletes it when it ends.

Stage 2 is stage 1 byte for byte.
Stage 2 has the same class files as stage 1, at the same paths, with the same bytes.
When they differ, the script names the first class that differs and ends with status 1.
A difference is a defect in the compiler under `compiler/`, and the fix goes there.
The script refuses with status `2` when `JAVA_HOME` names no JDK and when the seed is missing.

The script is the check of the bootstrap, because it compares the two stages itself.
The runner runs on stage 1, so each check of the runner is a check of stage 1.

## Properties

These hold, and each is checked:

1. A module that declares `main` at the one shape is written with an entry point a JVM starts
   at, and every other module is written without one.
2. Running a module leaves exactly the class files building it leaves.
3. A run ends with the low eight bits of the `Int` `main` gave back.
4. Every word after the file reaches the program unchanged, `--help` among them.
5. Stage 2 of the bootstrap is stage 1 byte for byte, and both stages pass `tests/spec`.
6. A checkout with no class file bootstraps with `JAVA_HOME` alone, and `bin/bux` then starts.
