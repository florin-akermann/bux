Rewrite a source file in canonical form.

There is exactly one canonical formatting of every Bux program, and source that is not
written in it does not compile. `bux fmt` is how a file gets there: it reads the file,
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

Formatting never changes what a program says: each item parsed from the canonical text is the
item parsed from the original, and only the order of the items can change.

Canonical form covers sequence too, and `bux fmt` repairs each order the compiler can compute.
It sorts the imports and puts them first. It moves a declaration written after a test above the
tests, and the tests keep the order they are written in. It moves a declaration below each
declaration that uses it, and two declarations that use each other keep the order they are
written in. Each item moves with the comments directly above it, and the comments above the
first item stay at the top of the file up to its first `///` or `// example:` line. A file
the resolver refuses keeps the order of its declarations, and `bux build` says why. A `match`
lists its arms in the order the type declares its variants, and `bux fmt` does not reorder
them, because that needs the types.

How a name is spelled is fixed the same way, and it is not rewritten. A function, a
parameter, a record field, and an imported module are snake_case; a type, a variant, and a type
parameter are PascalCase; an acronym is a word, so UserId compiles and UserID does not. A
declared name is two characters or more, and a function whose result is Bool begins is_, has_,
can_, or should_. What a thing is called is the author's decision, so `bux check` says what
canonical form spells it rather than spelling it.

A file that does not parse is reported as a diagnostic, and nothing is written.

Exit codes: 0 when the file is canonical or was made canonical, 1 when it does not parse, and
2 when it cannot be read or written.
