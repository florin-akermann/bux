# Reaching the console and the file system

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

## Properties

These hold and are checked with property-based tests:

1. A name either module declares has the type this spec gives it, wherever it is written.
2. A name either module does not declare is refused with `L0414`, naming the module and it.
3. A call of a name of either is a static call of that module's class and of nothing else.
4. Every JVM class either module reaches is one this spec names.
5. The only method of either that guards a span is an `extern` whose result is a `Result`.
