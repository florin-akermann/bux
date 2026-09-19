Report the first line of a source file that is not in canonical form.

`lumen check` answers the question the compiler asks before anything else: is this file
written the one way Lumen writes programs? It reads the file, prints the canonical text of the
program it holds, and compares the two. Nothing is written back.

A file that differs is reported by line: its number, what the line says, and what canonical
form writes there. `lumen fmt` is the command that fixes it.

A file that does not parse is reported as the parse error instead.

Exit codes: 0 when the file is in canonical form, 1 when it is not or does not parse, and 2
when it cannot be read.
