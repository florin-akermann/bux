# Lexer

The lexer turns source text into a sequence of tokens, each carrying a byte span into that text.
It is the first compiler phase; the parser (Item 002) consumes its output.
This spec covers the version 0.1 surface of `docs/design.md` and `docs/implementation.md` section 9.

## Intent

Lexing is total: every input string produces a token sequence, and the lexer never reports an error.
Input the language does not know is still tokenised, as an `Unknown` or `UnterminatedString` token.
The parser is the phase that turns such a token into a diagnostic.

Tokens carry only a kind and a span; the token's text is the source slice the span names.
No token is decoded here: the value of an integer or string literal is the parser's business.

## Token kinds

| Kind                 | Text                                                                    |
|----------------------|-------------------------------------------------------------------------|
| `Identifier`         | `[A-Za-z_][A-Za-z0-9_]*`, unless the word is a keyword                    |
| `Keyword(k)`         | one of the keywords below                                               |
| `Integer`            | `[0-9]+`                                                                |
| `String`             | `"`, then any characters with `\` escaping the next one, then `"`        |
| `Punct(p)`           | one of the operators and punctuation below                              |
| `Newline`            | `\n`                                                                    |
| `LineComment`        | `//` to the end of the line, excluding the line ending                  |
| `UnterminatedString` | a `"` whose closing `"` does not arrive before the line or input ends    |
| `Unknown`            | one character the language has no use for                              |

The keywords are the words the grammar reserves:
`fn`, `type`, `trait`, `instance`, `derive`, `let`, `var`, `if`, `else`, `for`, `in`, `match`,
`break`, `continue`, `return`, `import`, `extern`, `process`, `spawn`, `test`, `true`, `false`.
`_` is reserved alongside them: it is the discard, never an identifier, and `_x` is a name as ever.
A later grammar item that reserves a word adds it here first.

The punctuation, longest match first:
`==`, `!=`, `<=`, `>=`, `+=`, `&&`, `||`, `->`, `=>`,
`=`, `<`, `>`, `+`, `-`, `*`, `/`, `%`, `!`, `?`, `.`, `,`, `:`, `|`, `(`, `)`, `{`, `}`, `[`, `]`.
`+=` is the one compound assignment `docs/design.md` shows; others arrive with a design change.

A keyword and a punctuation each know the text that spells them, which is how a later phase
names one in a message; lexing that text yields that token back.

Spaces, tabs, and carriage returns separate tokens and are not tokens themselves.
A carriage return also ends a line, so CRLF input never puts a `\r` inside a token.
A newline is a token because the grammar is newline-sensitive, as Go's is.
A comment is a token because the formatter must carry comments through the round trip.

## Spans

A span is a byte offset and a length; its end is the offset plus the length.
Every span is non-empty, lies within the source, and begins and ends on a character boundary.
Tokens come in source order and do not overlap.
Every byte of the source not covered by a token is a space, a tab, or a carriage return.
So the tokens plus the blanks between them are the whole input, byte for byte.

## Behaviour

An identifier is the longest run of identifier characters; `x1` is one token, not `x` and `1`.
A word that spells a keyword is that keyword; `fn` is `Keyword(Fn)`, `fnord` is an identifier.
Identifiers are ASCII; a non-ASCII letter is an `Unknown` token covering that whole character.

An integer is the longest run of ASCII digits; `123abc` is `Integer` then `Identifier`.
A minus sign is always `Punct(Minus)`; negation is the parser's concern.

A string runs from its opening quote to the first unescaped closing quote on the same line.
Inside a string, a backslash makes the next character part of the string, whatever it is.
Which escapes are valid is decided when the parser decodes the literal, not here.
A string that reaches the end of its line or of the input first is one `UnterminatedString` token.
Its span stops before the line ending, so the newline is still its own token.

Punctuation prefers the longest match: `==` is one token, and `- >` is two tokens.
A character that starts no token, such as `@`, `#`, `$`, or a lone `&`, is one `Unknown` token.

The lexer accepts any string, including empty input, which lexes to no tokens.

## Executable examples

`tests/spec/lexer/<name>.bx` files are lexed and compared with the sibling `<name>.tokens` file.
Each line of the `.tokens` file is `<kind> <start>..<end> <text>`, one per token, in source order.
A keyword and a punctuation are named by the text that spells them, as in `Punct(->)`.
The text is quoted, and a quote, a backslash, a line break, a tab, and a return are escaped.
`tests/siblings.bx` walks that directory and names the failing file.
An example is a whole program held to `docs/specs/executable-examples.md`, not a fragment.

## The lexer written in Bux

`compiler/lexer.bx` is this lexer, written in Bux.
It is the first phase of the Bux compiler, which `docs/implementation.md` section 6 states.

A Bux string is read one UTF-16 code unit at a time, and a span still counts UTF-8 bytes.
So the lexer scans code units and adds up the UTF-8 width of each unit that a token covers.
A unit below 128 is one byte, and a unit below 2048 is two bytes.
Each half of a surrogate pair is two bytes, because the pair is one four-byte character.
Every other unit is three bytes.
An `Unknown` token covers a whole surrogate pair, so it covers one whole character.

A Bux token also holds its text, because a later phase cannot cut a string at a byte offset.
The parser reads a name, a number, and a string from that text.
A token kind, a keyword, and a punctuation derive `Eq`, so the parser compares kinds with `==`.

`lexer.listed(source)` writes one line for each token: the kind, a space, and `<start>..<end>`.
A keyword and a punctuation show the text that spells them, as `Keyword(fn)` and `Punct(->)` do.
Every other kind shows its name, as `Identifier` and `UnterminatedString` do.

`tests/lexing.bx` holds the lexer to the properties below, on drawn text.
`tests/siblings.bx` holds it to every `.tokens` file under `tests/spec`, token for token.

## Properties

These hold for arbitrary text and are checked by drawn properties in the runner:

1. Lexing never panics.
2. Spans are in order, non-overlapping, non-empty, and on character boundaries.
3. Every byte outside a span is a blank (space, tab, or carriage return); no span holds a `\r`.
4. Lexing is deterministic.
5. A sequence of token texts joined by single spaces lexes back to the same sequence of kinds.
