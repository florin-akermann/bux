Compile a source file and run the program it holds.

`bux run` does what `bux build` does and then hands the result to a JVM. The class files are
written under `target/` exactly as a build writes them, so a run leaves the same files behind and
nothing more. That includes the class of every module the program imports, because the program
reaches them while it runs. The class path of the program is that one `target/` directory, then
each Java archive that a `jar` line of the manifest states, in line order, then the archives of
each dependency. Nothing else is on it, and `bux help packages` states the `jar` line.

A program starts at `main`, which takes the words it was run with and gives back the status the
run ends with:

```text
import io

fn main(arguments: List<String>) -> Int {
    for argument in arguments {
        io.println(argument)
    }
    0
}
```

A module that declares `main` at that shape is a program; every other module is a library, and
running one is refused rather than guessed at. The whole signature is read: the one parameter is
`List<String>`, and the result is `Int`. The entry point a JVM starts at is written with the
module. The JVM gets `--enable-preview` because every class Bux writes is a value class, which
JDK 28 holds in preview.

A program reaches its library, its stated archives, and the `java.base` module of the JDK, and
nothing more of the JDK.
The JVM gets `--limit-modules java.base`, so no class of `java.naming`, `java.rmi`, or
`java.scripting` loads. It also gets `-Djdk.serialFilter=!*`, so it makes no object from
serialized bytes. `JAVA_TOOL_OPTIONS`, `JDK_JAVA_OPTIONS`, and `_JAVA_OPTIONS` are removed from
the environment of the program, because each can add an option such as `-javaagent` to its JVM.
`bux test` starts the JVM of its examples in the same way.

Every word written after the file goes to the program, in that order and unchanged, and nothing
else does:

```sh
bux run report.bx --wide notes.txt
```

`--wide` and `notes.txt` are what the program reads out of `arguments`. The name of the program
is not among them, and a command that writes no word after the file runs it with an empty list.

`--help` is one of those words too, so `bux run` has no help flag of its own: a word cannot
mean one thing to the program and another to the runner. Read this topic with `bux help run`.

The status the run ends with is the `Int` `main` gave back. A program with nothing to say gives
back `0`. A status is eight bits wide wherever the JDK runs, so a program is ended with the low
eight bits of that answer and nothing else: giving back `256` ends the run with `0`, and giving
back `-1` ends it with `255`.

A program reaches the console with `io`, the file system with `files`, and another program with
`programs`, three library modules the compiler carries rather than reading from a file:

```text
import files
import io

fn main(arguments: List<String>) -> Int {
    match files.read("notes.txt") {
        Ok(text) => io.print(text)
        Err(why) => io.eprintln(why)
    }
    0
}
```

`io.print` writes its text and nothing else, `io.println` writes it and then a line break,
`io.eprintln` writes it and a line break to standard error, and `files.read` gives back the whole
file as a `Result` the program must open, so a file that is not there is a case the program states
rather than a failure that ends it. `programs.run` takes a program and the list of arguments it is
given, starts it, reads what it wrote, and waits for it to end, and what it gives back holds the
code it ended with and both of the texts it wrote.

A program is compiled before the JDK is looked for, so a program that does not compile is told so
on a machine that could not have run it anyway.

The JDK comes from `JAVA_HOME` and from nowhere else. Searching `PATH` would run whichever JVM
happened to be first on it, and a build that is reproducible deserves a run that is too. An unset
`JAVA_HOME`, or one holding no `bin/java`, is reported as such.

What the program writes to standard output and to standard error, `bux run` passes through
untouched, and the status the program ends with is the status `bux run` ends with.

Exit codes before a program runs: 1 when the compiler refuses the program, and 2 when a file
cannot be read or written, the JDK is not found, or the module declares no `main` at that shape.
Once the program runs, the run ends with the status the program ended with, whatever it is. A
program the operating system stopped rather than let end is reported as 128.
