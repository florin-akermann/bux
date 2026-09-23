Rewrite a source file in canonical form.

There is exactly one canonical formatting of every Lumen program, and source that is not
written in it does not compile. `lumen fmt` is how a file gets there: it reads the file,
prints the canonical text of the program it holds, and writes that text back. A file that is
already canonical is left untouched.

The formatter has no options. Indentation, spacing, line breaks, and the placement of comments
are fixed by the language, so no two people and no two tools can disagree about them, and no
diff is ever a formatting diff.

Imports form one block: no blank line goes between two imports, and one blank line follows
the last of them. A blank line between two imports does not compile, and `bux fmt` takes it
out.

Every comment survives. A comment written at the end of a line moves to a line of its own
directly above it; a comment already alone on its line stays where it is.

Formatting never changes what a program says: the tree parsed from the canonical text is the
tree parsed from the original. That is also why `lumen fmt` does not put a file in order.
Canonical form covers sequence too — imports come first and sorted, a declaration is written
below what uses it, and a `match` lists its arms in the order the type declares its variants —
but where a declaration belongs is the author's decision, so `lumen check` says where it goes
rather than moving it there.

How a name is spelled is fixed the same way and rewritten no more than order is. A function, a
parameter, a record field, and an imported module are snake_case; a type, a variant, and a type
parameter are PascalCase; an acronym is a word, so UserId compiles and UserID does not. A
declared name is two characters or more, and a function whose result is Bool begins is_, has_,
can_, or should_. What a thing is called is the author's decision, so `lumen check` says what
canonical form spells it rather than spelling it.

A file that does not parse is reported as a diagnostic, and nothing is written.

Exit codes: 0 when the file is canonical or was made canonical, 1 when it does not parse, and
2 when it cannot be read or written.
