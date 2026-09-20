Compile a source file and run the program it holds.

`lumen run` does what `lumen build` does and then hands the result to a JVM. The class files are
written beside the source exactly as a build writes them, so a run leaves the same files behind
and nothing more.

A program starts at `main`, which takes nothing and gives back nothing:

```text
fn main() -> () {
    greet("world")
}
```

A module that declares it is a program; one that does not is a library, and running it is refused
rather than guessed at. The entry point a JVM starts at is written with the module, so running the
module class with `java --enable-preview` directly does exactly what `lumen run` does. The flag is
there because every class Lumen writes is a value class, which JDK 28 holds in preview.

A program reaches the console with `io` and the file system with `files`, two modules the compiler
supplies until a module can be loaded from source:

```text
import files

import io

fn main() -> () {
    match files.read("notes.txt") {
        Ok(text) => io.print(text)
        Err(why) => io.println(why)
    }
}
```

`io.print` writes its text and nothing else, `io.println` writes it and then a line break, and
`files.read` gives back the whole file as a `Result` the program must open, so a file that is not
there is a case the program states rather than a failure that ends it.

A program is compiled before the JDK is looked for, so a program that does not compile is told so
on a machine that could not have run it anyway.

The JDK comes from `JAVA_HOME` and from nowhere else. Searching `PATH` would run whichever JVM
happened to be first on it, and a build that is reproducible deserves a run that is too. An unset
`JAVA_HOME`, or one holding no `bin/java`, is reported as such.

What the program writes, `lumen run` passes through untouched, and the status the program ends
with is the status `lumen run` ends with.

Exit codes: 0 when the program ran and ended normally, 1 when the compiler refuses the program,
and 2 when a file cannot be read or written, the JDK is not found, or the module declares no
`main`. A program that fails while running ends with whatever status the JVM gives it, and one the
operating system stopped rather than let end is reported as 128.
