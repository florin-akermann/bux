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

## The generator

`1brc/src/create_measurements.bx` writes the input, so no program in another language writes it.

```text
bux run 1brc/src/create_measurements.bx <rows> <path>
```

It writes `<rows>` lines to the file at `<path>`, and it empties whatever that file held before.
`<rows>` is a whole number in decimal digits, from `0` up to 18 digits.
A line is `<station>;<reading>` and a line end, as the section on the input states.

The stations are the stations of the challenge, 412 names, each with its mean temperature.
The module holds them in one list literal of records, each a name and a mean in tenths.
A line draws a station, with the same chance for each, and then a reading of that station.
The reading in tenths is the mean of the station plus the sum of three draws.
Each of the three is a whole number from -100 to 100, with the same chance for each.
So the readings of a station spread around its mean with a standard deviation of about 10.0.
The challenge draws from a normal distribution with a standard deviation of 10.0, so the two agree.
A reading is held from -999 to 999 tenths, and it is written with one decimal, such as `-3.4`.

A draw is a linear congruential generator with Knuth's MMIX constants, as `tests/drawn.bx` has.
The module writes its own, because no program imports a module of `tests/`.
The first draw starts from the seed 1, so two runs with the same count write the same file.
A run of fewer rows writes the first lines of a run of more rows.

The file is written in parts of one million lines.
Each part is one `String`, and `files.append` writes it after the parts before it.
So a run of one billion lines holds one part in memory at a time.

Its exit codes are these:

- `0`: the program wrote the file.
- `2`: an argument is missing, the count is no whole number, or the file cannot be written.
  The program says why on standard error.

## The layout

- `1brc/src/main.bx` is the program, a module of its own; `bux build` writes `1brc/src/target/`.
- `1brc/src/create_measurements.bx` writes a file of measurements, a module of its own.
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

It also runs `bux test` over `1brc/src/create_measurements.bx`, whose two tests say the rest.
The first writes 5000 lines into the temporary directory, and runs `bin/bux run 1brc/src/main.bx`.
It holds the output to one entry for each station, in the order of the names, with one decimal.
Then it deletes the file.
The second writes 300 lines twice, and holds the two files to the same text.
The first test starts `bin/bux` from the current directory, so a run of it starts at the root.

## What dogfooding found

Three gaps came up, and question 12 of `docs/principles.md` judges each.

- A whole number read off text is a `for` loop over `strings.at`, so the program writes it.
- An iterator over a map is the list of names the program holds, so nothing lands.
- A sort is the loop `sorted` in `src/command.bx` writes, and the program writes it again.
  `docs/specs/library.md` lands a function that a reader writes twice, so `list.sorted` lands.

The generator found two more.

- No `for` loop appends to a file, so `files.append` lands, as `docs/specs/io.md` states.
- `strings.join` adds each part to the text it has so far, which copies that text again each time.
  A part of one million lines is about 14 MB, so such a join copies terabytes.
  The generator joins two neighbours at a time until one part is left, so each pass copies once.
  That loop is written once, so it stays in the generator, and `strings.join` does not change.

`docs/implementation.md` section 7 records the wall time of a run over one billion lines.
