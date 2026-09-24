# Diagnostics

Every refusal the compiler makes is a diagnostic: a code, a message, the source it points at, and
often a line saying what to do about it.
`docs/implementation.md` section 8 sets the voice; this spec sets the layout, the codes, and the
exit codes.
No diagnostic mentions the JVM, a class file, or a stack frame.

## Intent

A reader who sees a diagnostic learns three things at once: what is wrong, where, and what to do.
The code is the stable handle on the first of those.
A spec or a test cites `L0105`, never the prose, which is then free to be reworded.

A code is assigned once and never reused, even after the diagnostic it named is removed.
Every code has one home: the place that raises it, and one explanation beside it.

## The layout

A rendered diagnostic is the message, the location, the source it points at, and the help:

```text
error[L0105]: comparisons do not chain
  --> demo.bx:2:5

  2 |     a < b < c
    |     ^^^^^^^^^

help: compare twice and join the two with `&&`
```

The first line is `error[`, the code, `]: `, and the message.
A message is one line and never ends in a full stop.
The second line is two blanks, `--> `, and the file, the line, and the column joined by colons.
A blank line follows, then the source line, then the caret row.
A diagnostic that has something to advise ends with a blank line, `help: `, and that line.
Every rendering ends with a newline.

The source line and the caret row share a gutter: two blanks, the line number, and one blank.
The caret row writes blanks where the number goes, and both rows continue with `| `.
The line and the column are one-based, as an editor counts them.
The column counts characters rather than bytes.
A tab before the span is written back as a tab, so the two rows line up however wide a tab is.

Only the line the span starts on is shown, however far the span runs.
A span that starts past the end of the file is shown on its last line.
A span that ends on a later line carets the rest of its first line and says where it ends:

```text
  2 |     a <
    |     ^^^ this runs on to line 3
```

## The codes

A code is `L` and four digits, grouped by the phase that raises it.
The grammar writes `L01xx`, canonical form `L02xx`, loading and name resolution `L03xx`, and
inference `L04xx`.
Exhaustiveness writes `L05xx`, and what a build asks of a module it compiles writes `L06xx`.
Code generation writes `L07xx`, and the shape of a process writes `L08xx`.

The long form of a code is the file in `compiler/explanations/` named after it.
`command.explanation_of` reads that file as a class-path resource, so there is one copy of it.

A code that stops being raised is taken out whole: the number, the long form, and the paragraph
that stated it.
The number is not given to anything else afterwards, so a gap in the run is a code that was
retired and nothing more.

The grammar raises these, in `compiler/parser.bx`:

- `L0100` — the grammar expected one thing and the source wrote another.
- `L0101` — a string has no closing quote.
- `L0102` — a character is not part of the language.
- `L0103` — a number does not fit in a whole number.
- `L0104` — a backslash is followed by something that is not an escape.
- `L0105` — comparisons are chained.
- `L0106` — brackets nest deeper than the parser descends.
- `L0107` — something other than a name is written on the left of `=` or `+=`.
- `L0108` — a call names some of its arguments and not others.
- `L0109` — a statement binds a name with `:=`, which is not part of Bux.
- `L0110` — a test is written inside a body, where only a statement is written.

Canonical form raises these, in `compiler/format.bx`:

- `L0200` — the file is not in canonical form.
- `L0201` — an import is after a declaration or out of sort, or a declaration is after a test.
- `L0202` — a declared name is spelled some way other than the one canonical form spells it.
- `L0203` — a declared name is an initial rather than a word a reader can look for.

One code covers all three ways a file departs from whitespace form, because they are one problem
and `bux fmt` is the one answer to it.
Order is its own code because its message names an item and the place it belongs.
`bux fmt` is the answer to it too, and its help says so.

Naming has two codes for the same reason, and one each because the two have different answers: a
miscased name has the spelling canonical form gives it, and an initial has a word only the author
knows.
`L0413` is with inference rather than here because the result it reads is the one inference
settled, which `docs/specs/naming.md` states.

Name resolution raises these, in `compiler/resolver.bx`:

- `L0300` — nothing in scope has this name.
- `L0301` — a module declares the same name twice.
- `L0302` — a declaration or a binding hides a name that is already in scope.
- `L0303` — a declaration is written above something that uses it.
- `L0304` — a name that is not a value, such as a function or a module, is written as one.
- `L0305` — an assignment names something other than a `var` binding.
- `L0308` — a trait already has an instance for the type a second instance names.
- `L0309` — an instance does not write exactly the methods its trait declares.
- `L0310` — something that is not a trait is written where a trait belongs.
- `L0311` — a trait is written where a type belongs.
- `L0312` — a derive names a trait no type derives, or a type this module does not declare.
- `L0313` — a type or a pattern is reached through a name that is no module.
- `L0314` — a name that binds is written inside an or-pattern, which binds nothing.
- `L0318` — an instance writes an argument of its type that is no type parameter it declares.
- `L0319` — two tests of one module have the same name.
- `L0320` — an import is read by nothing.
- `L0321` — a binding or a parameter is read by nothing.

Loading raises these, in `compiler/modules.bx`, before any module is resolved:

- `L0306` — an import names a module neither a file beside it nor a package it reaches holds.
- `L0307` — a ring of imports, which leaves the modules in it no order to be compiled in.
- `L0315` — a manifest states something other than `package`, `version`, a `depends`, and a `jar`.
- `L0316` — a directory named as a package holds no manifest, so there is no package there.
- `L0317` — two files claim the module name an import writes, so one build would hold both.

Type inference raises these, and `compiler/refusal.bx` words them:

- `L0400` — a type met a type it does not match.
- `L0401` — a call passes more or fewer arguments than the function takes.
- `L0402` — a field is reached that the type reached through does not have.
- `L0403` — a type would have to contain itself.
- `L0404` — a record is built without one of the fields it declares.
- `L0405` — a record is written with one of its fields given a value twice.
- `L0406` — `==` or `!=` is written between two values of a type that has no `Eq`.
- `L0407` — a division is written with a divisor of zero, which has no answer.
- `L0408` — a statement leaves a value behind and nothing takes it.
- `L0409` — a call passes its arguments in order where the declaration repeats a type.
- `L0410` — an argument is named something other than the parameter it is passed for.
- `L0411` — a call names the arguments of something that has no parameter names.
- `L0412` — a parameter is a bare `Bool` outside a function that is about `Bool`.
- `L0413` — a function whose result is `Bool` is named for a command rather than a question.
- `L0414` — a module does not declare the name reached inside it.
- `L0415` — a declared type holds a value of itself, around a ring that comes back to it.
- `L0418` — a trait method is used at a type that has no instance of that trait.
- `L0419` — a parameter of a method a trait declares states no type.
- `L0420` — a whole number does not fit the type it is written at.
- `L0421` — a bound of an `IntegerLiteral` instance is not one whole number.
- `L0422` — a type derives a trait and holds a value of a type that has no instance of it.
- `L0424` — a generic of another module is constrained by a trait that stays in that module.
- `L0425` — an `extern` signature names a type no Java member takes or gives back.
- `L0426` — an `extern` states something that is no Java name.
- `L0427` — a derive names a type an `extern type` declares, whose contents are the JVM's.
- `L0428` — an `extern method` or an `extern new` reaches a class, and its signature names none.
- `L0429` — an `extern` writes a width where there is no `Int` to widen to or to narrow from.
- `L0430` — an `extern new` gives back a type an `extern type` named an interface.
- `L0431` — an `extern` narrows a parameter and gives back something other than an `Option`.
- `L0432` — a program writes `Option<()>`, or inference gives an expression a type that holds it.
- `L0433` — a call is written with a value in front of the name, where a dot reads a field.
- `L0434` — an `extern` names a class in no stated archive and not on the list, or a member out.
- `L0435` — a result, a field, a variant, or a call gives back or holds an `extern` type.
- `L0436` — a function or a trait method leaves a parameter type or its result out.

`compiler/escapes.bx` decides `L0435` after inference, because a call is read at its settled type.
`compiler/types.bx` decides `L0436` after inference, so its help spells the settled signature.

Exhaustiveness raises these, in `compiler/exhaustiveness.bx`:

- `L0500` — a `match` leaves a value of the type it matches unanswered.
- `L0501` — a `match` lists its arms in an order the type does not declare its variants in.

`bux build` raises this one, in `compiler/command.bx`, for each hole `compiler/holes.bx` finds:

- `L0600` — a hole is still in the program, and a hole has nothing to compile.

`bux build`, `bux run`, and `bux test` raise these, in `compiler/command.bx`:

- `L0601` — a function a module declares at the top level states no example.
- `L0602` — an example is written where nothing carries one.

`bux test` raises these alone, in `compiler/command.bx`:

- `L0603` — an example a module states did not hold when it was run.
- `L0604` — a module declares the name a run of its examples reaches for.

All three are reported together rather than one at a time, unlike every code above them.
A reader answering them is answering a list, and a list of one would not be that list.

Every command that takes a file raises this one, in `compiler/command.bx`, before it compiles:

- `L0605` — the file named on the command line is no Bux source: its name does not end in `.bx`.

`bux build`, `bux run`, and `bux test` raise these in `compiler/archives.bx`.
Each is raised before a class is written:

- `L0606` — an archive that a `jar` line names is not there.
- `L0607` — an archive has a hash other than the hash that its `jar` line states.
- `L0608` — the manifest of an archive has a `Class-Path`, which names more archives.
- `L0609` — a file that a `jar` line names is no Java archive.

`bux run` and `bux test` raise this one, in `compiler/command.bx`, before they compile:

- `L0610` — the directory of the module holds `:`, which splits the class path of the run.

`bux build`, `bux run`, and `bux test` raise these, in `compiler/jvm.bx`, for the first
class that the writer cannot write:

- `L0700` — a function is too large to compile as one function.
- `L0701` — a text or a name takes more than 65535 bytes.
- `L0702` — two parts of one program have one name, or two class names differ only in case.
- `L0703` — the compiler did not write a function, which is a defect of the compiler.

Name resolution raises the first two of these, in `compiler/resolver.bx`.
Inference raises the same two for a process reached through a module, whose surface it reads.
The shape check raises the next eight, in `compiler/processes.bx`, before inference.
The declarations raise `L0810`, in `compiler/declared.bx`.
The process phase raises `L0811` after the declarations, in `compiler/processes.bx`.
`docs/specs/concurrency.md` states the shape that each one holds.

- `L0800` — `spawn` names no process, or names one without a call.
- `L0801` — a process is called as a function or held as a value, without `spawn`.
- `L0802` — the first function of a process is not `start`.
- `L0803` — the second function of a process is not `receive`.
- `L0804` — a process declares a function after `receive`.
- `L0805` — a function of a process declares a type parameter.
- `L0806` — a function of a process leaves the type of a parameter or of its result unwritten.
- `L0807` — `receive` does not take exactly two values, a state and a message.
- `L0808` — the body of `receive` is not one `match` on the message.
- `L0809` — an arm of that `match` is not one call or one name.
- `L0810` — `receive` does not take the state `start` gives and give `Next` of that state.
- `L0811` — what `start` takes, the state, or the message of a process holds an `extern` type.

## As data

`bux check --json <file>` writes the refusal as data rather than as a page to read.

A tool that wants to apply the compiler's own edit should not have to read the rendered form back.
The rendered form is written for a person, and rewording it is free; the data form is written for
a program, and every field of it is named.

One diagnostic is one JSON object on one line, and the line ends with a newline.
`bux check` stops at the first refusal, so there is at most one line.
A file the compiler accepts is nothing at all: no output, and the exit code says it went well.

```text
{"file":"demo.bx","code":"L0200","message":"this line is not in canonical form","span":{"start":9,"len":10},"help":"canonical form writes `    a < b`","fix":{"start":0,"len":22,"text":"fn f() {\n    a < b\n}\n"}}
```

`file` is the path as it was given on the command line.
`code` is the code, written the way `bux explain` takes it.
`message` is the same one line the rendered form opens with.
`span` is where in the file the diagnostic points, as byte offsets: `start` and `len`.
A byte offset is what an editor and a language server both work in, and a line and a column are
both worked out from it, so the data form carries the one and not the other.

`help` is the advice, and it is left out when there is none rather than written as `null`.
`fix` is left out the same way, and it is there only where the compiler knows the edit to make.

The data form goes to standard output, because with `--json` the diagnostic is what was asked for.
Standard error stays empty, so a run can be piped straight into a tool.
That is about refusals, which are the only thing the data form is for.
A file that cannot be read at all is not a diagnostic about a program: it is said on standard
error and exits `2`, with the flag exactly as without it.

## The edit

`fix` is an edit: replace the `len` bytes at `start` with `text`.

Applying it is a byte splice and nothing more, so a tool needs no knowledge of the language.
An edit need not cover the bytes `span` covers: `span` is where the reader is pointed, and `fix`
is what answers the refusal, which is not always the same place.

Canonical form is the one thing the compiler carries an edit for, because it is the one refusal
whose answer the compiler already knows: `bux fmt` writes exactly that text.
What a name should have been and which variant a `match` is missing are the author's to decide.
So those carry a `help` and no `fix`.
Where an item belongs has an answer too, and its refusal carries a `help` that names `bux fmt`.

That edit is the whole file, and it is what `bux fmt` would write where every item is in place.
One line at a time would not do: a file with a line too many has every line after it disagreeing,
and rewriting one of them where it stands leaves text that is no longer a program.
One edit that replaces the file always lands, and applying it is `bux fmt` by another route.

Applying the edit answers the refusal it came with, not every refusal the file holds.
A file whose imports are also out of order is told about canonical form first, because the text is
held to canonical form before the order is looked at; applying the edit and checking again then
reports `L0201`, which carries no edit of its own and a help that names `bux fmt`.

## `bux explain`

`bux explain <code>` prints the long form of one code and exits `0`.
The long form says what the diagnostic means, why the language refuses it, and what to write
instead, at more length than a `help:` line has room for.
It lives beside the catalogue, one file per code, so a code and its explanation never drift.

A code the compiler cannot raise is reported on standard error, and the run exits `2`.

## Exit codes

A command exits `0` when it has nothing to refuse, `1` when it refuses the program it was given,
and `2` when it cannot do its job at all.
A file that cannot be read and a code that does not exist are both `2`: neither is about a
program.
Diagnostics go to standard error, and only what was asked for goes to standard output.

`bux run` is the one command that can end with a status of somebody else's choosing.
Once the program it was given runs, the run ends with the status that program ended with, which
`docs/specs/run.md` states.
Both `1` and `2` are given before a program runs, so neither is ever a status a program chose.

## The diagnostics written in Bux

`compiler/command.bx` renders a diagnostic in Bux.
It writes the layout above: the `error[` line, the location, the source line, and the carets.
It counts the column in characters and keeps a tab before the span as a tab.
A span that runs on to a later line gets the note that names the line it ends on.
The data form is the same one line of JSON, with the same escapes, and `fix` where there is one.
The one edit is the edit above: the whole file, replaced by the text `bux fmt` writes.
`explain` reads the long form of a code as a class-path resource.

`tests/rendering.bx` holds the renderer to the properties below, on drawn sources and spans.
`tests/commanded.bx` holds `check`, `check --json`, and `explain` to golden answers.

## Properties

These hold and are checked by drawn properties in the runner:

1. A rendering names a line and a column that lie inside the source.
2. A rendering opens with its code and ends with a newline, for any span at all.
3. A data form is one line, whatever text the diagnostic holds.
4. Text a data form writes reads back as the text it was given.
5. Applying the fix of a file that departs from canonical form makes it its own canonical text.

`tests/commanded.bx` holds `explain` of each code that has a long form to the text of that file.
