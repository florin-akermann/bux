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
io.eprintln(text: String) -> ()
files.read(path: String) -> Result<String, String>
```

`io.print` writes `text` and nothing else.
`io.println` writes `text` and then a line break.
Both go to the standard output a JVM starts with, and neither gives anything back.

`io.eprintln` writes `text` and then a line break, on standard error rather than standard output.
Standard error is where a program says what went wrong, so what it says there stays apart from
the answer it writes on standard output, and a reader of either channel reads one thing.
`docs/specs/run.md` has `lumen run` pass both through unchanged.

`files.read` reads the whole file at `path` and gives back its text.
Reading can fail, so it gives back a `Result`, as `docs/design.md` section 5 requires of one.
`Ok` holds the text, and `Err` holds what went wrong, as a line of text.
A caller opens the `Result` to reach the text, so a missing file is a case, not a failure.
The file is read as UTF-8, and a file that is not UTF-8 is an `Err` like any other.

Each module declares the `extern` declarations those are written over, and `io` declares the type
one of them names.
`docs/specs/api-surface.md` makes every top-level name public, and neither module is exempt, so
`io.out`, `io.err`, `io.put`, `io.put_line`, `io.PrintStream`, `files.read_whole`,
`files.as_a_path`, `files.named`, `files.File`, and `files.Path` are each reachable by name.
That is what writing these two in Lumen costs, and it is the price of their being modules rather
than a table inside the compiler.
A program that wants a line written writes `io.println`, and the rest is how that is built.

## How each is reached

A module is reached only through its own name, so a file writes `import io` before `io.println`.
An import naming neither module names a file, which `docs/specs/modules.md` states.

`io.write` is refused where it is written, because `io` declares no `write`.
That is the ordinary rule for a module, which `docs/specs/modules.md` states, and neither of
these two adds anything to it.

## Writing a file, listing a directory, and reading a variable

`files` read one file whole and did no more.
A compiler writes a file too, and makes, lists, and removes a directory, and reads a variable.
`environment` is a module of its own, because one module holds one thing and a variable is no
file.

```text
files.write(path: String, text: String) -> Result<String, String>
files.listed(path: String) -> Result<List<String>, String>
files.made(path: String) -> Result<String, String>
files.removed(path: String) -> Result<Bool, String>
environment.read(name: String) -> Option<String>
```

`files.write` writes `text` as the whole of the file at `path`, and empties whatever was there.
`files.listed` gives the name of everything the directory at `path` holds, in order.
`files.made` makes one directory at `path`, and makes no parent of it.
`files.removed` removes what is at `path`, and gives back whether anything was there to remove.
`environment.read` gives the variable called `name`, and `None` where nothing set it.

Every failure is an `Err` that holds one line of text, and nothing here throws through a program.
The line is what the JVM said of itself, except for the two members that say nothing, below.
A caller opens the `Result` to reach the answer, so a directory that is not there is a case rather
than a failure, exactly as `files.read` already has it.

`files.write` and `files.made` give back `path` in the `Ok`, so a caller writes the next step
over what the last one said.
`Ok` carries the path rather than nothing at all for a second reason.
`docs/specs/doc-examples.md` asks every function for an example that is an expression and is true,
and no expression over a `Result<(), String>` is one.

`environment.read` gives an `Option` and not a `Result`.
A process started without a variable is an ordinary case, and there is nothing that went wrong to
report.

## Which members these are written over

`docs/specs/interop.md` states what crosses: `Bool`, `Int`, `String`, a type an `extern` names,
and `Option` or `Result` around one of those.
An array crosses neither way, and a member that gives nothing back leaves an `Ok` nothing to
carry, so four plain members are out of reach for one of those two reasons.

| the member                            | what it holds that does not cross        |
|---------------------------------------|------------------------------------------|
| `java.io.File.list`                   | gives back an array of `String`          |
| `java.nio.file.Files.writeString`     | takes an array of `OpenOption`           |
| `java.nio.file.Files.createDirectory` | takes an array of `FileAttribute`        |
| `java.nio.file.Files.delete`          | gives nothing back for an `Ok` to carry  |

Each is written over a member that is reachable instead.

`files.write` opens a `java.io.PrintWriter` on the path, writes the text, and closes it.
A `PrintWriter` throws nothing while it writes and answers `checkError` instead, so `write` asks
it and builds the `Err` itself, naming the path.

`files.made` asks `java.io.File.mkdir`, which answers `true` or `false` and says no more, so the
`Err` of `made` is the library's own wording and names the path.

`files.removed` is `java.nio.file.Files.deleteIfExists`, which gives back a `Bool` and throws what
it cannot do.
`files.removed` hands that `Bool` on unchanged: `Ok(true)` says something was there, and
`Ok(false)` says nothing was.

`files.listed` is `java.nio.file.Files.list`, whose `java.util.stream.Stream` an `extern type`
names as it names any other class.
A `List` crosses a boundary as a parameter and never as a result, which `docs/specs/interop.md`
states, so the names are not handed over as one.
`files` reads them one at a time and pushes each onto a list of its own with `list.push`, which
`docs/specs/library.md` states, so the list a caller reads is one the library built.
`files` imports `list` for that one name, as `process` does for the same one.
Loading hands `list` over below `files`, as it does for any module an import names.
A failure anywhere in that walk is an `Err`, and a caller is never handed part of a listing.

The entries come back sorted.
`java.nio.file.Files.list` states no order, and a listing a program compares, or a build that is
repeated, needs one.
Each entry is a path, so `files.listed` reads the last part of it and gives the name alone.

A `java.util.stream.Stream` reads the directory as it is walked, and throws where that read fails,
so `files.listed` reads the whole stream out into a `java.util.List` first.
That one step is an `extern` declared with a `Result`, and the walk over the list that follows it
throws nothing.
The stream is let go whether the reading worked or did not, which gives the directory back to the
file system.

`environment.read` is `java.lang.System.getenv`, which gives back `null` where nothing set the
variable, and `docs/specs/interop.md` maps that `null` to `None`.

`environment` is a class of its own, as `io` and `files` each are, and a call of
`environment.read` is a static call of that class.
The JVM classes these reach are `java.lang.String`, `java.lang.System`, `java.lang.Object`,
`java.io.PrintWriter`, `java.io.File`, `java.nio.file.Path`, `java.nio.file.Files`,
`java.nio.charset.Charset`, `java.nio.charset.StandardCharsets`, `java.util.stream.Stream`,
`java.util.List`, and `java.util.Iterator`.
A class file is built of three more that no declaration here writes.
`java.lang.Boolean` is what the `Bool` an `Ok` carries is boxed as.
`java.lang.Throwable` and `java.lang.AssertionError` are what a guard and an unreachable arm are
made of.
`docs/specs/codegen.md` states each of the three.
A `List` is held as `lumen.List`, which the build writes, so it is no JVM class that these reach.
`java.util.stream.Stream`, `java.util.List`, and `java.util.Iterator` are each an interface, which
the declaration says with the word `docs/specs/interop.md` gives it.
`java.lang.Object` is what an entry of a listing is held as, because `java.util.Iterator.next`
gives one back, and asking it for its text is how the path is read.

`docs/specs/api-surface.md` makes every top-level name public, so each `extern` declaration and
each step written beside these is reachable by name, as `files.read_whole` already is.
`files` adds `sent`, `names_within`, `every_name`, `each_bare_name`, `opened`, `put`, `closed`,
`failed`, `make`, `delete`, `entries`, `in_order`, `all_of`, `one_by_one`, `released`,
`has_another`, `next_value`, `as_text`, `bare_name`, `Writer`, `Entries`, `Held`, `Walk`, and
`Anything`.
`environment` declares `read` and nothing else.
That is what writing these in Bux costs, and a program that wants a file written writes
`files.write`.

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
`io.out` or `io.err` reads off `java.lang.System`.
The two streams are the one difference between `io.println` and `io.eprintln`, which write over
the same `io.put_line`.

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
process.run(command: String, arguments: List<String>) -> Result<Finished, String>
```

`command` names the program, and `arguments` holds what it is given, in the order it reads them.
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
A command that names no program and a read that fails are each a case the `Result` carries, and
the caller opens it as it opens the one `files.read` gives back.

The program reads the standard input the program that started it reads.
A pipe of its own is what it would otherwise read, and nothing ever writes on that one, so a
program that asks for a line would wait for a line no one is going to send.

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
`process.Finished`, `process.whole_command`, `process.whole`, `process.of_command`,
`process.reading_from`, `process.inherited`, `process.started`, `process.output_of`,
`process.errors_of`, `process.ended`, `process.over`, `process.delimited`, `process.token`,
`process.closed`, `process.nothing_at_all`, `process.ProcessBuilder`, `process.Process`,
`process.InputStream`, `process.Scanner`, and `process.Redirect` are each reachable by name.
That is what writing the module in Bux costs, as it is for `io` and `files`.
A program that wants a program started writes `process.run`, and the rest is how that is built.

`run` builds the one `java.util.List` a `java.lang.ProcessBuilder` is built from: it starts from
a list holding `command` alone and pushes each value of `arguments` after it, with `list.push` in
a `for` loop.
A Bux `List<String>` crosses the boundary as a `java.util.List` of what it holds.
`docs/specs/interop.md` states the rule that lets it, and `docs/specs/codegen.md` states the copy.
`run` then builds a `java.lang.ProcessBuilder`, points its standard input at the one this program
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
5. Every method of one that guards a span is an `extern` whose result is a `Result`.
