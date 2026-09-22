# Reaching the console, the file system, and another program

## Intent

A program that can work something out and show nobody is not much of a program.
Version 0.1 has no way to write a line and none to read a file, and `io` and `files` give it one.
`docs/design.md` section 16 states how a module is reached; this spec states what these two hold.

They are library modules: the compiler carries the Lumen source of each and loads it as it loads
`list` and `strings`, which `docs/specs/library.md` states.
What they hold is a call of the JVM's own, and `docs/specs/interop.md` states the declaration that
names one.
Loading reads each out of the source the compiler carries, so a file of either name beside the
importing one is never read; `docs/specs/modules.md` states that.

## What each module declares

```text
io.print(text: String) -> ()
io.println(text: String) -> ()
files.read(path: String) -> Result<String, String>
```

`io.print` writes `text` and nothing else.
`io.println` writes `text` and then a line break.
Both go to the standard output a JVM starts with, and neither gives anything back.

`files.read` reads the whole file at `path` and gives back its text.
Reading can fail, so it gives back a `Result`, as `docs/design.md` section 5 requires of one.
`Ok` holds the text, and `Err` holds what went wrong, as a line of text.
A caller opens the `Result` to reach the text, so a missing file is a case, not a failure.
The file is read as UTF-8, and a file that is not UTF-8 is an `Err` like any other.

Each module declares the `extern` declarations those are written over, and `io` declares the type
one of them names.
`docs/specs/api-surface.md` makes every top-level name public, and neither module is exempt, so
`io.out`, `io.put`, `io.put_line`, `io.PrintStream`, `files.read_whole`, `files.as_a_path`,
`files.named`, `files.File`, and `files.Path` are each reachable by name.
That is what writing these two in Lumen costs, and it is the price of their being modules rather
than a table inside the compiler.
A program that wants a line written writes `io.println`, and the rest is how that is built.

## How each is reached

A module is reached only through its own name, so a file writes `import io` before `io.println`.
An import naming neither module names a file, which `docs/specs/modules.md` states.

`io.write` is refused where it is written, because `io` declares no `write`.
That is the ordinary rule for a module, which `docs/specs/modules.md` states, and neither of
these two adds anything to it.

## The errors

```text
L0414  a module does not declare the name reached inside it
```

Every module is held to that rule, which `docs/specs/modules.md` states.

## What the bytes look like

Nothing here is visible from Lumen, and `docs/specs/codegen.md` states the rest of it.

Each module is a class of its own, and a call of `io.println` is a static call of that class, the
way a call of `strings.join` is.
Each `extern` those are written over is a static method of the same class, whose body is the
member and the mapping around it; `docs/specs/interop.md` states what that body is.

`io.put` and `io.put_line` call the method of `java.io.PrintStream` they name, on the stream
`io.out` reads off `java.lang.System`.

`files.read` builds a `java.io.File`, asks it for a `java.nio.file.Path`, and reads that whole with
`java.nio.file.Files.readString`.
The two steps that can throw are each declared with a `Result`, so each is a guarded method of its
own, which every `extern` already is.
Those are the only places the compiler catches anything, and each catches only to build the
`Result` its declaration promises.
A Lumen value is never asked what it is, and the one thing asked here is not a Lumen value.

## Starting a program

`bux run` and `bux test` each start a `java`, and a compiler written in Bux has to start one too.
`process` is a library module of its own, beside `io` and `files`, and it declares one function a
program reaches for.

```text
process.run(command: List<String>) -> Result<Finished, String>
```

`command` holds the program and every argument it is given, in that order.
`run` starts that program, reads everything the program wrote, and waits for it to end.
`Ok` holds a `Finished`, which is a record `process` declares:

```text
type Finished = {
    code: Int
    output: String
    errors: String
}
```

`code` is the code the program ended with.
`output` is what the program wrote on its standard output, and `errors` is what it wrote on its
standard error.
Each of the two is the text the program wrote and nothing else, so a program that wrote nothing
leaves an empty one.
`Err` holds a line of text, and it is a command that never started at all.
A program that starts and then fails is an `Ok` whose `code` says so, because it ran.

Nothing here is partial.
A command that names no program, an empty command, and a read that fails are each a case the
`Result` carries, and the caller opens it as it opens the one `files.read` gives back.

The program reads the standard input the program that started it reads.
A pipe of its own is what it would otherwise read, and nothing ever writes on that one, so a
program that asks for a line would wait for a line no one is going to send.

### Why the command is one list

`java.lang.ProcessBuilder` is built from one `java.util.List` holding the program and its
arguments.
A Bux `List<String>` is that list already, which `docs/specs/codegen.md` states, so it crosses the
boundary as itself; `docs/specs/interop.md` states the rule that lets it.

The other shape is a `run` that takes the program and the arguments apart.
It needs a value put in front of a list, and nothing in the library writes that yet, which
`docs/specs/library.md` says of `list.push`.
A signature the library cannot write is not the smaller change, so `run` takes the one list the
JVM starts a program from, and takes it whole.

### What reading the output amounts to

`run` reads the standard output whole, then the standard error whole, then waits for the code.
Reading either one ends when the program closes it, which is when the program ends, so a read is
what waits.
Each read closes what it read, so a run gives every handle it took back.

One case is beyond that, and this spec states it rather than leaving a reader to find it.
The operating system holds only so much of what a program writes before that program waits for a
reader, and `run` reads the standard error only once the standard output is closed.
A program that writes more than that much on its standard error, and closes neither stream,
waits for a reader that is reading the other one, and `run` waits with it.
Reading the two at once is what answers that, and it needs the `spawn` of version 0.4.
Until then `run` is for a program that writes less than one such hold on its standard error.

### What `process` declares

`docs/specs/api-surface.md` makes every top-level name public, and `process` is not exempt, so
`process.Finished`, `process.whole`, `process.of_command`, `process.reading_from`,
`process.inherited`, `process.started`, `process.output_of`, `process.errors_of`,
`process.ended`, `process.over`, `process.delimited`, `process.token`, `process.closed`,
`process.nothing_at_all`, `process.ProcessBuilder`, `process.Process`, `process.InputStream`,
`process.Scanner`, and `process.Redirect` are each reachable by name.
That is what writing the module in Bux costs, as it is for `io` and `files`.
A program that wants a program started writes `process.run`, and the rest is how that is built.

`run` builds a `java.lang.ProcessBuilder`, points its standard input at the one this program
reads, starts it, and reads each stream with a `java.util.Scanner`.
The scanner is given a delimiter no text holds, so the whole stream is one token, and the scanner
is closed once that token is read.
A stream nobody wrote on holds no token at all, and `process.nothing_at_all` is the stream that
states that case in an example.
`start`, `waitFor`, and `next` each give back a `Result`, so each is a guarded method of its own,
which every `extern` already is.

## Properties

These hold and are checked with property-based tests:

1. A name a module here declares has the type this spec gives it, wherever it is written.
2. A name a module here does not declare is refused with `L0414`, naming the module and it.
3. A call of a name of one is a static call of that module's class and of nothing else.
4. Every JVM class a module here reaches is one this spec names.
5. The only method of one that guards a span is an `extern` whose result is a `Result`.
