# Grammar

The parser turns the lexer's tokens into an untyped abstract syntax tree.
It is the second compiler phase; name resolution (Item 006) consumes its output.
This spec covers the version 0.1 surface of `docs/design.md` and `docs/implementation.md` section 9.

## Intent

The AST is untyped and immutable.
Every node carries the span of the source it was parsed from, and no node is ever mutated.
A later phase produces a new representation rather than annotating this one.

Parsing is fallible, and it stops at the first error.
The parser reports one error with a span and, where a fix is obvious, a `help:` line.
Error recovery is not part of version 0.1; one error at a time is what `lumen check` needs.

The parser decides nothing a later phase can decide better.
It does not know which names exist, which are types, or which are variants.
`User { id: id }` and `user { name: "Bob" }` parse to the same node; Item 006 tells them apart.

## What is not in the grammar

**There are no anonymous functions.**
`fn` not followed by a name is a parse error, not a value.
This is the one place the parser enforces a language rule rather than a shape.

Comments and blank lines are not part of the grammar.
The parser drops comment tokens, and the AST holds none.
How the formatter carries a comment through the round trip is decided in Item 003.

A type declaration declares a record or variants; there is no type alias, which `docs/design.md`
never writes, and `type Ids = List<Int>` is therefore a parse error.

Nothing from a later version is parsed: no `trait`, `derive`, `extern`, `spawn`, or effect arrow.
The lexer reserves no word for them, so each reads as an ordinary identifier and fails in place.

## Newlines

The grammar is newline-sensitive, as Go's is, and follows Go's rule for which newlines matter.

A newline is **significant** when the token before it can end a statement, a field, or an arm:
an identifier, an integer, a string, `true`, `false`, `break`, `continue`, `return`, `?`,
`)`, `]`, `}`, or `>`.
Every other newline is dropped before parsing begins, so an expression continues across a line
break after an operator, a comma, `=`, `:=`, `+=`, `->`, `=>`, `|`, or an opening bracket.

`>` is in that list because it closes a type argument list, which is how `ids: List<Int>` ends.
It is also the greater-than operator, so that one operator is never the last token of a line.
Canonical form never writes one there, and every other operator still continues a line below it.
Comments are dropped first, so a trailing comment never changes whether a newline is significant.
Consecutive significant newlines collapse: a blank line separates nothing extra.

A significant newline terminates the item, statement, field, variant, or match arm it follows.
Significant newlines before a closing `}` or `)` are allowed and terminate nothing.

`else` follows its `}` on the same line; a newline between them ends the `if` statement.

## Grammar

Uppercase and lowercase spellings are not distinguished; `Name` below is any identifier.

```text
program        := { item }

item           := import | type_declaration | function

import         := "import" Name

type_declaration := "type" Name [ type_parameters ] "=" type_definition
type_parameters  := "<" Name { "," Name } ">"
type_definition  := record_type | variant | { "|" variant }
record_type      := "{" { Name ":" type } "}"
variant          := Name [ "(" type { "," type } ")" | record_type ]

type           := Name [ "<" type { "," type } ">" ] | "(" ")"

function       := "fn" Name [ type_parameters ] "(" [ parameters ] ")" [ "->" type ] block
parameters     := parameter { "," parameter }
parameter      := Name [ ":" type ]

block          := "{" { statement } "}"
statement      := binding | assignment | "return" [ expression ]
                | "break" | "continue" | for | expression
binding        := Name ":=" expression | "var" Name "=" expression
assignment     := expression ( "=" | "+=" ) expression
for            := "for" [ Name "in" expression | expression ] block

expression     := or
or             := and { "||" and }
and            := comparison { "&&" comparison }
comparison     := sum [ ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) sum ]
sum            := product { ( "+" | "-" ) product }
product        := unary { ( "*" | "/" | "%" ) unary }
unary          := [ "!" | "-" ] postfix
postfix        := primary { "(" [ arguments ] ")" | "." Name | "?" }
arguments      := expression { "," expression }
primary        := Name [ record_literal ] | Integer | String | "true" | "false"
                | "(" ")" | "(" expression ")" | if | match
record_literal := "{" [ field_value { "," field_value } ] "}"
field_value    := Name ":" expression

if             := "if" expression block [ "else" ( block | if ) ]
match          := "match" expression "{" { match_arm } "}"
match_arm      := pattern "=>" expression

pattern        := Name [ "(" pattern { "," pattern } ")" | "{" Name { "," Name } "}" ]
                | Integer | String | "true" | "false"
```

A comparison does not chain: `a < b < c` is a parse error, as it is in Go.
Every other binary operator is left-associative.

A list the grammar writes with at least one element is not accepted empty.
`List<>`, `fn f<>()`, `Failed()`, and the pattern `P {}` each name what was wanted instead.
A call, a parameter list, and a record literal are the three lists the grammar writes as optional.

Brackets nest at most 32 deep, which no program a person or the formatter writes comes near.
The budget is what makes "parsing never panics" true of generated input: the parser reports an
error where it would otherwise recurse until the stack is gone.

A record literal is not parsed where a block would follow an expression, as in Go.
Those places are the condition of an `if`, the header of a `for`, and the scrutinee of a `match`.
Writing one there needs parentheses: `if (user { active: true }).active { … }`.

## Literals

The lexer hands over the text of a literal; the parser decodes it.

An integer literal decodes to a signed 64-bit value, and one too large to fit is an error.
A `-` straight before the digits is part of the number, so the smallest whole number can be
written; a `-` before anything else, a blank included, is the prefix operator.
`-5` is one literal and `- 5` is a negated `5`, so the tree always says what the source says.
A postfix operator then applies to the literal: `-5.abs()` reads the field of `-5`.
A leading zero decodes fine and is not canonical form, so Item 003's gate is what rejects it.

A string literal decodes its escapes: `\"`, `\\`, `\n`, `\t`, and `\r`.
Any other escape is an error naming the character that followed the backslash.
An `UnterminatedString` token is an error, and so is an `Unknown` token, wherever either appears.

## Spans

Every node's span covers exactly the source it was parsed from, opening and closing bracket
included, and lies within the span of the node that holds it.
Item spans are in source order and do not overlap.
An error's span is non-empty and lies within the source, and names the token that failed —
or, at the end of input, the last token.

## Errors

A parse error reads `expected <what>, found <what was there>`, in the voice of
`docs/implementation.md` section 8.
The expectation names a thing the reader writes, never a parser state: `a name`, `a type`,
`an expression`, `a pattern`, `a function name`, `the end of the line`,
`an import, a type, or a function`, or the exact token, as in `` `)` ``.
The found part names what is there the same way, or `the end of the file`.

Seven failures are not about which token was found, and have their own words:

| Error                | Message                                        | Help                               |
|----------------------|------------------------------------------------|------------------------------------|
| `fn` without a name  | expected a function name, found `` `(` ``      | every function has a name          |
| unterminated string  | this string has no closing quote               | add a closing `"`                  |
| unknown character    | this character is not part of the language     | (none)                             |
| unknown escape       | `` `\q` `` is not an escape                     | the escapes the language knows     |
| number too large     | this number does not fit in a whole number     | the largest whole number           |
| chained comparison   | comparisons do not chain                       | compare twice and join with `&&`   |
| nesting too deep     | this nests too deeply to parse                 | how deep brackets may nest         |

## Executable examples

`tests/spec/parser/<name>.lm` files are parsed and compared with a sibling expectation file.
A `<name>.ast` file holds the parse tree, one node per line, `<indent><node> <start>..<end>`.
A `<name>.error` file holds `<start>..<end> <message>`, then a `help: <text>` line if there is one.
A `.lm` file has exactly one of the two, and the parser crate's tests name the failing example.

## Properties

These hold and are checked with property-based tests:

1. Parsing never panics, on any input.
2. Parsing is deterministic.
3. An error's span is non-empty and lies within the source.
4. A generated well-formed program parses, and its item spans are ordered and non-overlapping.

The round trip `parse(print(ast)) == ast` needs a printer, and lands with it in Item 003.
