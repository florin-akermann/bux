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
reported as 128.

## Properties

These hold, and each is checked:

1. A module that declares `main` at the one shape is written with an entry point a JVM starts
   at, and every other module is written without one.
2. Running a module leaves exactly the class files building it leaves.
3. A run ends with the low eight bits of the `Int` `main` gave back.
4. Every word after the file reaches the program unchanged, `--help` among them.
