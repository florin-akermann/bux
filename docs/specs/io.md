# Reaching the console and the file system

## Intent

A program that can work something out and show nobody is not much of a program.
Version 0.1 has no way to write a line and none to read a file, and `io` and `files` give it one.
`docs/design.md` section 16 states how a module is reached; this spec states what these two hold.

They are supplied modules: the compiler declares them, the way it declares the prelude.
It does so because what they hold is a call of the JVM's own, which no Lumen source states yet.
Loading looks for no file for either, so a file of that name beside the importing one is never
read; `docs/specs/modules.md` states that.

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

## How each is reached

A module is reached only through its own name, so a file writes `import io` before `io.println`.
An import naming neither module names a file, which `docs/specs/modules.md` states.

A supplied module holds exactly the names above.
`io.write` is refused where it is written, because `io` is supplied and declares no `write`.
That is all these modules add; everything else about them is the ordinary rule for a module.

## The errors

```text
L0414  a module does not declare the name reached inside it
```

A module loaded from a file is held to the same rule, which `docs/specs/modules.md` states.

## What the bytes look like

Nothing here is visible from Lumen, and `docs/specs/codegen.md` states the rest of it.

`io.print` and `io.println` read the standard output off `java.lang.System`.
Each calls the method of `java.io.PrintStream` with the same name as the Lumen one.
Reading a static field is the one instruction this needs that nothing else did.

`files.read` is a static method of `lumen.Files`, a class the compiler writes as it writes the
prelude, and a call of `files.read` is a call of it.
It is written with a module that reads a file, and with no other.
The read is a method of its own because a guarded span begins with an empty stack: a throw
discards everything below it, and a read may be written wherever an expression is.

The method builds a `java.io.File`, asks it for a `java.nio.file.Path`, and reads that whole.
`java.nio.file.Files.readString` is what reads it.
The read is guarded: anything thrown is caught, and the `Err` holds what it says of itself.
It is the one place version 0.1 catches anything, and only to give the caller the `Result`.
A Lumen value is never asked what it is, and the one thing asked here is not a Lumen value.

## Properties

These hold and are checked with property-based tests:

1. A name a supplied module declares has the type this spec gives it, wherever it is written.
2. A name a supplied module does not declare is refused with `L0414`, naming the module and it.
3. A call of a supplied name is lowered reaching only the classes this spec names.
4. `files.read` leaves a `Result` on the stack down every path out of it, the caught one included.
5. No method a module writes guards a span, because the one guarded span is a method of its own.
