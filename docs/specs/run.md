# Running a program

## Intent

`lumen run <file>` compiles a module and runs it.
It is the shortest way from a source file to the thing the source file does, and it is what an
executable example uses to prove a program does not only compile but works.

`docs/design.md` section 11 says where a program starts: `main`, taking nothing and giving back
nothing.
A module that declares `main` is a program; one that does not is a library.

## What running amounts to

`lumen run` does what `lumen build` does and then hands the result to a JVM.
The class files are written beside the source, exactly as `lumen build` writes them, so a run
leaves the same artefacts a build does and nothing more.

The JVM is started on the module class, in the directory the classes were written to.
Running the module class directly with `java` does the same thing, because the entry point is
part of what is written and not something the runner supplies.

The program's own output is the runner's output: what it writes, the runner passes through
unchanged, and the status it ends with is the status `lumen run` ends with.

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
error: demo.lm: a module is run through `fn main() -> ()`, which this one does not declare
error: JAVA_HOME is not set, and running a program needs the JDK it names
error: /opt/nothing/bin/java: JAVA_HOME names no JDK
```

A module declaring `main` with any other signature is the same as not declaring it, because
`fn main() -> ()` is the one shape a program starts at.
The message names that shape rather than the word alone, so a module declaring the wrong one is
told what to write instead of sent looking for what is already there.

## Exit codes

```text
0  the program ran and ended normally
1  the compiler refused the program
2  a file could not be read or written, the JDK was not found, or there is no `main`
```

A program that fails while running ends with whatever status the JVM gives it, which is the
program's answer and not the compiler's.
A program the operating system stopped rather than let end has no status of its own, and is
reported as 128, which is neither of the two the compiler ends with.

## Properties

These hold, and each is checked:

1. A module that declares `main` is written with an entry point a JVM starts at, and one that
   declares no `main` is written without one.
2. Running a module leaves exactly the class files building it leaves.
