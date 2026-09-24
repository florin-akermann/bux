# The One Billion Row Challenge

`1brc/src/main.bx` answers the One Billion Row Challenge, and it is the second dogfood program.

## Intent

`example/main.bx` is the program a newcomer reads, and `1brc/src/main.bx` is the program measured.
The challenge reads one file of one billion lines, and writes one line of output.
So it measures what a program of everyday Bux costs: a loop over text, a map, and processes.
It is written in Bux over `library/` alone, and it declares no `extern` of its own.
A gap it finds is judged by question 12 of `docs/principles.md`, as each dogfood gap is.

## The input

Each line of the file is `<station>;<reading>`, and a line end is the byte 10.
A station name is UTF-8 of 1 to 100 bytes, and it holds no `;` and no line end.
A reading is from `-99.9` to `99.9`, with one decimal, such as `-3.4` or `12.0`.
The file holds 10 000 station names at most.
The last line of the file can end without a line end.
A line that holds no `;` is no reading, and the program skips it.

## The output

The program writes one line on standard output, and nothing else.

```text
{Abha=-23.0/18.0/59.2, Abidjan=-16.2/26.3/67.3, Abéché=-10.0/29.4/69.0}
```

Each station shows its lowest reading, its mean reading, and its highest reading.
Each of the three has one decimal, and a slash is between each two.
The stations are in the order of their names, as `<` orders two `String` values.
A comma and a space are between each two stations, and braces are around all of them.

A half rounds toward positive infinity, as `Math.round` rounds, which the challenge states.
So a mean of `-0.15` is `-0.1`, and a mean of `0.15` is `0.2`.
A mean that rounds to zero is `0.0`, and never `-0.0`.

## How the program counts

A reading is held in tenths as an `Int`, so `-3.4` is `-34`, and no floating type is needed.
A station has a summary: its lowest and highest reading, the total of its readings, and a count.
A billion readings of `99.9` total less than 10^12 tenths, so the total fits in an `Int`.

The mean in tenths is the floor of `total / count + 1/2`, which is how `Math.round` rounds.
That is the floor of `(2 * total + count) / (2 * count)`, so the program holds no fraction.
`/` on `Int` truncates toward zero, and gives `None` where the divisor is zero.
So the program writes a floored division that steps a negative quotient down by one.
A summary starts at its first reading, so its count is never zero.

## Exit codes

- `0`: the program wrote the line.
- `2`: no file is named, or the file cannot be read, and the program says why on standard error.

With no argument, the program writes `usage: bux run 1brc/src/main.bx <file of measurements>`.
Every word after the first argument is not read.

## How the file is split

`main` asks `files.size` for the size of the file, and `environment.processors` for a count.
It spawns one worker `process` for each processor, and gives each one part of the file.
Part `i` of `n` runs from byte `size * i / n` up to byte `size * (i + 1) / n`.
A line belongs to the part that holds its first byte.
So a worker starts at the first line that starts at or after the start of its part.
It ends where the first line that starts at or after the end of its part starts.
Each line is thus read by one worker exactly, whatever the count of workers and of lines.
A part that holds no first byte of a line reads nothing.

A worker reads its lines in slices of 1 MiB with `files.read_between`.
A slice is cut after its last line end, so no line is split between two slices.
The last slice of a part is read whole, because a file need not end with a line end.
`read_between` gives one char for each byte, which is ISO-8859-1.
So `;`, the digits, the sign, and the line end are each one char, whatever the name holds.

A worker holds a `map.Map<String, Summary>` and the list of the names it read, in that order.
A name stays as its bytes in the worker, one char for each byte.
The worker tallies its part in `start`, and gives the tally as its state.
`main` sends each worker `Finish`, takes each tally with `ended`, and merges them by the names.
It gives each name back its UTF-8 with `strings.from_utf_8`, once for each station of each worker.
Then it sorts the names with `list.sorted` and writes the line.

An error of a read stops the worker with that error as its state.
`main` then writes the error on standard error, and ends with the status 2.

## The layout

- `1brc/src/main.bx` is the program, a module of its own; `bux build` writes `1brc/src/target/`.
- `1brc/tests/samples/<name>.txt` is an input, and `<name>.out` is the output it expects.

## The samples

- `utf-8-names`: names with bytes above 127, such as `Zürich` and `İzmir`.
- `negative-half`: a mean of `-0.15`, which is `-0.1`, and a mean of `-0.05`, which is `0.0`.
- `one-station`: one line, so every worker but one reads nothing.
- `uneven-rows`: 97 lines, which no count of processors from 2 to 96 divides.
  Its last line has no line end.

`tests/started.bx` runs `1brc/src/main.bx` over each sample, as it runs `example/main.bx`.
It holds the status to `0` and the output to the `.out` file of the sample.
It also runs the program with no argument, and holds the status to `2`.
It runs `bux test` over the module, so every `// example:` line of it holds.

## What dogfooding found

Three gaps came up, and question 12 of `docs/principles.md` judges each.

- A whole number read off text is a `for` loop over `strings.at`, so the program writes it.
- An iterator over a map is the list of names the program holds, so nothing lands.
- A sort is the loop `sorted` in `compiler/command.bx` writes, and the program writes it again.
  `docs/specs/library.md` lands a function that a reader writes twice, so `list.sorted` lands.

`docs/implementation.md` section 7 records the wall time of a run over one billion lines.
