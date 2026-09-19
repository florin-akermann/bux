Rewrite a source file in canonical form.

There is exactly one canonical formatting of every Lumen program, and source that is not
written in it does not compile. `lumen fmt` is how a file gets there: it reads the file,
prints the canonical text of the program it holds, and writes that text back. A file that is
already canonical is left untouched.

The formatter has no options. Indentation, spacing, line breaks, and the placement of comments
are fixed by the language, so no two people and no two tools can disagree about them, and no
diff is ever a formatting diff.

Every comment survives. A comment written at the end of a line moves to a line of its own
directly above it; a comment already alone on its line stays where it is.

Formatting never changes what a program says: the tree parsed from the canonical text is the
tree parsed from the original.

A file that does not parse is reported as the parse error, and nothing is written.

Exit codes: 0 when the file is canonical or was made canonical, 1 when it does not parse, and
2 when it cannot be read or written.
