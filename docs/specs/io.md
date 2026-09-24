# Reaching the console, the file system, and another program

## Intent

A program that can work something out and show nobody is not much of a program.
Version 0.1 has no way to write a line and none to read a file, and `io` and `files` give it one.
`docs/design.md` section 16 states how a module is reached; this spec states what these two hold.

They are library modules: the compiler carries the Bux source of each and loads it as it loads
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
`docs/specs/run.md` has `bux run` pass both through unchanged.

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
That is what writing these two in Bux costs, and it is the price of their being modules rather
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
`files` imports `list` for that one name, as `programs` does for the same one.
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
`environment.processors` reaches `java.lang.Runtime`, which the section below states.
A class file is built of three more that no declaration here writes.
`java.lang.Boolean` is what the `Bool` an `Ok` carries is boxed as.
`java.lang.Throwable` and `java.lang.AssertionError` are what a guard and an unreachable arm are
made of.
`docs/specs/codegen.md` states each of the three.
A `List` is held as `bux.List`, which the build writes, so it is no JVM class that these reach.
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

## Writing bytes

A class file holds bytes and not text, and a compiler written in Bux writes class files.
So `files` writes bytes with one more function.

```text
files.write_bytes(path: String, bytes: List<Int>) -> Result<String, String>
```

`files.write_bytes` writes `bytes` as the whole of the file at `path`, one byte for each value.
It empties whatever was there, and it gives back `path` in the `Ok`, as `files.write` does.
Each value is a byte from 0 to 255, and a program splits a larger number into bytes itself.
A value outside 0 to 255 is an `Err` that names the value, and the file is not opened.
A byte is never written as a replacement for a value that does not fit.

No member of the JVM that writes a byte can cross the boundary.
A byte array crosses neither way, and `java.io.OutputStream.write` gives nothing back.
An `Ok` needs something to carry, so no `extern` can give back `()` for that member.
So `files.write_bytes` writes on the `java.io.PrintWriter` that `files.write` opens too.
It opens that writer with the ISO-8859-1 encoding.
That encoding writes each char from 0 to 255 as the one byte of the same value.
`strings.as_latin_1` names that encoding, beside `strings.as_utf_8`.
`strings` declares both and their type `strings.Encoding`, and `files` uses those three.
Each byte is a string of one char, which `java.lang.Character.toString(int)` gives.
That member takes an `int`, so its declaration `strings.one_char` narrows and gives an `Option`.
The writer is closed and asked `checkError`, as `files.write` asks it.
`files` adds `write_bytes` and `bytes_sent` for this, and `strings` declares `one_char`.
`strings` declares each member of `java.lang.Character` that the library and the compiler reach.
So `files` reaches one more JVM class, `java.lang.Long`.
`strings` reaches `java.lang.Character`.
The refusal of a value shows that value, and `java.lang.Long.toString` shows a number.

`tests/spec/io/bytes_written.bx` writes bytes, reads them back as text, and shows them.
`docs/specs/codegen.md` states the one caller in the compiler, which writes each class file.

## Appending to a file

`files.write` writes a file whole, and a file of one billion rows holds about 13 GB.
No `String` holds that much, so a program that writes such a file writes it one part at a time.

```text
files.append(path: String, text: String) -> Result<String, String>
```

`files.append` writes `text` as UTF-8 after what the file at `path` holds.
Where no file is at `path`, it makes the file first, so the first part is written as the rest are.
It gives back `path` in the `Ok`, as `files.write` does.
A file that cannot be opened or written is an `Err` that holds what the JVM said.
A program that wants a file of its own parts alone empties it first with `files.write(path, "")`.

`java.nio.file.Files.writeString` takes an array of `OpenOption`, so it does not cross.
A `java.io.PrintWriter` opened to append takes a `java.io.Writer`, and no subtype crosses.
So `files.append` opens a `java.io.FileOutputStream` with its second argument `true`.
That constructor opens the file to append, and it makes the file where there is none.
The stream gives its `java.nio.channels.FileChannel`, which writes at the end of the file.
`strings.encoded` writes the text as UTF-8 into a `java.nio.ByteBuffer`.
The channel writes from the buffer, and it gives the count of bytes it took as an `int`.
A write can take fewer bytes than are left, so `files` writes until the buffer holds no more.
A write that takes no byte is an `Err`, so no loop waits on a file that takes no more.

The stream is closed after the writes, whether they worked or did not.
A close gives nothing back, so its `extern` holds no `Result`, as the close of a read holds none.
The stream holds no buffer of its own, so each byte reached the file before the close.

`files` adds `append`, `appended_and_let_go`, `drained`, `opened_to_append`, `channel_to`,
`written_from`, `shut`, and `Sink` for this.
So `files` reaches one more JVM class, `java.io.FileOutputStream`.
`tests/spec/io/file_appended.bx` writes a line, appends two, and reads the file back.
`1brc/src/create_measurements.bx` is the first caller, which `docs/specs/billion-rows.md` states.

## Reading bytes

A build hashes each Java archive that a manifest states, and a hash is over bytes, not text.
So `files` reads bytes with one more function.

```text
files.read_bytes(path: String) -> Result<List<Int>, String>
```

`files.read_bytes` reads the whole file at `path`, and gives back one value for each byte.
Each value is from 0 to 255, in the order of the file.
A file that cannot be read is an `Err` that holds what went wrong, as `files.read` gives.

A byte array crosses neither way, so the file is read as text in the ISO-8859-1 encoding.
That encoding reads each byte as the one char of the same value, and every byte is a char.
`files.read_whole_as` is `java.nio.file.Files.readString` with an encoding.
`files.each_byte` and `files.values_of` turn each char into its value with `strings.at`.
So `files` imports `strings`, and it reaches no JVM class that it did not reach already.
`compiler/archives.bx` is the one caller, and `docs/specs/packages.md` states it.

## Reading a part of a file

`files.read` reads a file whole into one `String`, and a JVM `String` holds 2^31 chars at most.
A file of one billion rows holds about 13 GB, so no read of it whole can work.
A program that reads a large file reads it one part at a time, so `files` gives two more.

```text
files.size(path: String) -> Result<Int, String>
files.read_between(path: String, from: Int, to: Int) -> Result<String, String>
```

`files.size` gives the count of bytes that the file at `path` holds.
It is `java.nio.file.Files.size`, which gives a `long`, and the file is not opened.
A file that is not there, or that the file system cannot measure, is an `Err`.

`files.read_between` gives the bytes from `from` up to `to`, not including `to`.
Each byte is the one char of the same value, as `files.read_bytes` reads it in ISO-8859-1.
So the length of the text is the count of bytes, and a caller finds a line end by its offset.
A part cut at any byte can split a UTF-8 sequence, so a decoded read could not say where it ends.
`strings.from_utf_8` decodes a name that a caller cuts out of a part, which
`docs/specs/library.md` states.

A `to` past the end of the file gives the bytes up to the end, and a `from` past it gives `""`.
A negative `from`, or a `to` below `from`, is an `Err` whose message names the two offsets.
A part longer than 2147483647 bytes is an `Err` that names the length, before the file opens.
That count is the most that one JVM buffer holds, because a buffer counts in an `int`.
Every failure of the read is a `Result`, and the close below is the one step outside that.

`files.read_between` asks `files.size` first, and it reads no further than the end.
So a `to` far past the end asks for no buffer larger than what the file holds.
It opens a `java.io.FileInputStream` and asks it for its `java.nio.channels.FileChannel`.
The channel reads into a `java.nio.ByteBuffer` at a position, and a read reads no byte before it.
A read can give fewer bytes than the buffer has room for, so `files` reads until the buffer is full.
A read at the end gives `-1`, which also ends the reads.
`java.nio.charset.Charset.decode` then reads the buffer as ISO-8859-1 into a `java.nio.CharBuffer`.
Its text is the part, so no array crosses the boundary in either direction.
Each step that throws is an `extern` with a `Result`, as the steps of `files.listed` are.

The stream is closed after the read, whether the read worked or did not.
A close gives nothing back, so its `extern` holds no `Result`, which `docs/specs/interop.md` states.
A close of a file opened only to be read loses no byte, because the part was read before it.
A close that fails still throws through the program, and no `extern` can guard it today.
Such a failure is the operating system failing to let a descriptor go, which a read seldom meets.

`files` adds `size`, `read_between`, and the steps and declarations that each is written over.
So `files` reaches four more JVM classes: `java.io.FileInputStream`,
`java.nio.channels.FileChannel`, `java.nio.ByteBuffer`, and `java.nio.CharBuffer`.
`tests/spec/io/part_read.bx` writes a file, reads it in parts, and shows the parts.

## The errors

```text
L0414  a module does not declare the name reached inside it
```

Every module is held to that rule, which `docs/specs/modules.md` states.

## What the bytes look like

Nothing here is visible from Bux, and `docs/specs/codegen.md` states the rest of it.

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
A Bux value is never asked what it is, and the one thing asked here is not a Bux value.

## Starting a program

`bux run` and `bux test` each start a `java`, and a compiler written in Bux has to start one too.
`programs` is a library module of its own, beside `io` and `files`, and it declares one function a
program reaches for.

```text
programs.run(command: String, arguments: List<String>) -> Result<Finished, String>
```

`command` names the program, and `arguments` holds what it is given, in the order it reads them.
`run` starts that program, reads everything the program wrote, and waits for it to end.
`Ok` holds a `Finished`, which is a record `programs` declares:

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

### What `programs` declares

`docs/specs/api-surface.md` makes every top-level name public, and `programs` is not exempt.
So each of these names is reachable:

- `programs.Finished`, `programs.whole_command`, `programs.whole`, and `programs.of_command`.
- `programs.reading_from`, `programs.inherited`, `programs.started`, and `programs.output_of`.
- `programs.errors_of`, `programs.waited_for`, `programs.over`, and `programs.delimited`.
- `programs.token`, `programs.closed`, and `programs.nothing_at_all`.
- `programs.ProcessBuilder`, `programs.Running`, `programs.InputStream`, and `programs.Scanner`.
- `programs.Redirect`, and the members the section below adds.

That is what writing the module in Bux costs, as it is for `io` and `files`.
A program that wants a program started writes `programs.run`, and the rest is how that is built.

`run` builds the one `java.util.List` a `java.lang.ProcessBuilder` is built from: it starts from
a list holding `command` alone and pushes each value of `arguments` after it, with `list.push` in
a `for` loop.
A Bux `List<String>` crosses the boundary as a `java.util.List` of what it holds.
`docs/specs/interop.md` states the rule that lets it, and `docs/specs/codegen.md` states the copy.
`run` then builds a `java.lang.ProcessBuilder`, points its standard input at the one this program
reads, starts it, and reads each stream with a `java.util.Scanner`.
The scanner is given a delimiter no text holds, so the whole stream is one token, and the scanner
is closed once that token is read.
A stream nobody wrote on holds no token at all, and `programs.nothing_at_all` is the stream that
states that case in an example.
`start`, `waitFor`, and `next` each give back a `Result`, so each is a guarded method of its own,
which every `extern` already is.

## What the compiler and its tests stand on

`docs/implementation.md` section 3 has every program reach the platform through the library.
The compiler and its tests are programs, so each `extern` they call is declared in `library/`.
Each is declared once, and a class that two modules reach has one `extern type` in one of them.
A test of `tests/conventions.bx` refuses an `extern` in a file outside `library/` and `tests/spec/`.

`files` declares what the compiler asks of a path, each a member of `java.io.File`:

```text
files.is_there  files.is_file  files.is_directory  files.is_readable
files.has_made_all  files.parent_path  files.canonical_path  files.path_separator
```

`files.named` gives the `files.File` each of these takes, and `files.make` makes one directory.
`files` reads a part of a file into the `strings.Bytes` and `strings.Chars` that `strings` declares.

`environment.property` is `java.lang.System.getProperty`, and a property nothing set is `None`.
`environment.processors` is `java.lang.Runtime.availableProcessors`, which is never below one.
`clock.nanoseconds` is `java.lang.System.nanoTime`, whose difference in one run is a time.

`programs` declares what the compiler and its tests do to a program before and after its start:

```text
programs.inheriting  programs.within  programs.input_from  programs.output_to
programs.errors_to  programs.variables_of  programs.removed  programs.held_as_constant
programs.as_object  programs.is_ended_within  programs.stopped  programs.seconds
programs.Variables  programs.Constant  programs.TimeUnit
```

`programs.removed` takes the name of a variable as `files.Anything`, which is `java.lang.Object`.

`strings` declares the members of `java.lang.Character` that a module reaches.
They are `strings.one_char`, `strings.is_alphabetic`, and `strings.general_category`.
Each takes an `int`, so each narrows its argument and gives back an `Option`.

`jars` opens a Java archive and reads its manifest and its entries, over `java.util.jar.JarFile`.
Its members are `opened`, `manifest_of`, `main_attributes`, `value_of`, `entry_of`, and `closed`.
`bits` gathers, spreads, and turns the 64 bits of an `Int`, over `java.lang.Long`.
Its members are `gathered`, `spread`, and `turned`, which `compiler/digest.bx` calls.

## Properties

These hold and are checked by drawn properties, each a test of `tests/`:

1. A name a module here declares has the type this spec gives it, wherever it is written.
2. A name a module here does not declare is refused with `L0414`, naming the module and it.
3. A call of a name of one is a static call of that module's class and of nothing else.
4. Every JVM class a module here reaches is one this spec names.
5. Every method of one that guards a span is an `extern` whose result is a `Result`.
6. The parts of a drawn file, read one after another and joined, are what `files.read_bytes` gives.
