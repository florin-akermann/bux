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

A trait declares signatures and an instance writes bodies for them, which `docs/specs/traits.md`
states; a trait is written over one type parameter and an instance is for a type written by name.

An `extern` declaration names one member of one Java class and writes no body, which
`docs/specs/interop.md` states; `field`, `static`, `method`, and `new` are read only after
`extern`, and each is an ordinary identifier anywhere else.
A `method` declares at least one parameter, because its receiver is the first of them, and a
`field` declares none; both are the shape above rather than a refusal.
Every one of them writes its result, because there is no body for inference to read one off.

A `process` holds functions and nothing else, and the parser takes any number of them.
The type phase holds them to the one shape that `docs/specs/concurrency.md` states.
`spawn` takes one postfix operand, so `spawn Counter(0)` is one expression before any operator.
The parser takes any operand after `spawn`, and name resolution refuses one that is no process.

A width after the kind says what the member's descriptor gives back, `int` before a parameter's
name says the member takes one there, and `interface` after `type` says the class is one.
Each is an ordinary identifier everywhere else, including as the name of the declaration itself.
Telling a word from a name is the one place the grammar needs a second token: each is the word
where a name follows it and the name where `(`, `:`, `,`, `)`, or `=` does.

Nothing from a later version is parsed: no `spawn` and no effect arrow.
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

item           := import | type_declaration | trait | instance | derive | function
                | extern_type | extern | process

import         := "import" Name

type_declaration := "type" Name [ type_parameters ] "=" type_definition
type_parameters  := "<" Name { "," Name } ">"
type_definition  := record_type | variant | { "|" variant }
record_type      := "{" { Name ":" type } "}"
variant          := Name [ "(" type { "," type } ")" | record_type ]

type           := Name [ "<" type { "," type } ">" ] | "(" ")"

trait          := "trait" Name "<" Name ">" "{" signature { signature } "}"
signature      := "fn" Name "(" [ parameters ] ")" [ "->" type ]
instance       := "instance" Name "<" Name ">" "{" function { function } "}"
derive         := "derive" Name { "," Name } "for" Name
process        := "process" Name "{" { function } "}"

extern_type    := "extern" "type" [ "interface" ] Name "=" String
extern         := "extern" ( extern_field | extern_static | extern_method | extern_new )
extern_field   := "field" [ width ] Name "(" ")" "->" type "=" String
extern_static  := "static" [ width ] Name "(" [ taken ] ")" "->" type "=" String
extern_method  := "method" [ width ] Name "(" taken ")" "->" type "=" String
extern_new     := "new" [ width ] Name "(" [ taken ] ")" "->" type
taken          := extern_parameter { "," extern_parameter }
extern_parameter := [ "int" ] parameter
width          := "int" | "char"

function       := "fn" Name [ constrained_parameters ] "(" [ parameters ] ")" [ "->" type ] block
constrained_parameters := "<" constrained { "," constrained } ">"
constrained    := Name [ ":" constraint { "+" constraint } ]
constraint     := Name "<" type ">"
parameters     := parameter { "," parameter }
parameter      := Name [ ":" type ]

block          := "{" { statement } "}"
statement      := binding | assignment | discard | "return" [ expression ]
                | "break" | "continue" | for | expression
binding        := Name ":=" expression | "var" Name "=" expression
assignment     := Name ( "=" | "+=" ) expression
discard        := "_" "=" expression
for            := "for" [ Name "in" expression | expression ] block

expression     := or
or             := and { "||" and }
and            := comparison { "&&" comparison }
comparison     := sum [ ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) sum ]
sum            := product { ( "+" | "-" ) product }
product        := unary { ( "*" | "/" | "%" ) unary }
unary          := [ "!" | "-" ] unary | "spawn" postfix | postfix
postfix        := primary { "(" [ arguments ] ")" | "." Name | "?" }
arguments      := expression { "," expression }
                | named_argument { "," named_argument }
named_argument := Name ":" expression
primary        := Name [ record_literal ] | Integer | String | "true" | "false"
                | "(" ")" | "(" expression ")" | written_list | if | match
written_list   := "[" [ expression { "," expression } ] "]"
record_literal := "{" [ field_value { "," field_value } ] "}"
field_value    := Name ":" expression

if             := "if" expression block [ "else" ( block | if ) ]
match          := "match" expression "{" { match_arm } "}"
match_arm      := pattern "=>" expression

pattern        := alternative { "|" alternative }
alternative    := Name [ "(" pattern { "," pattern } ")" | "{" Name { "," Name } "}" ]
                | Integer | String | "true" | "false" | "_"
```

A comparison does not chain: `a < b < c` is a parse error, as it is in Go.
Every other binary operator is left-associative.

A list the grammar writes with at least one element is not accepted empty.
`List<>`, `fn f<>()`, `Failed()`, and the pattern `P {}` each name what was wanted instead.
A call, a parameter list, a record literal, and a written list are the four the grammar writes
as optional.

A call names all of its arguments or none of them, which the two alternatives above say.
A name followed by `:` opens a named argument, so the first argument settles which list follows.
`rename(from: old, new)` and `rename(old, to: new)` are each `L0108`.
Which of the two forms a call may use is type inference's to say, in `docs/specs/arguments.md`.

Brackets nest at most 32 deep, which no program a person or the formatter writes comes near.
The budget is what makes "parsing never panics" true of generated input: the parser reports an
error where it would otherwise recurse until the stack is gone.

A record literal is not parsed where a block would follow an expression, as in Go.
Those places are the condition of an `if`, the header of a `for`, and the scrutinee of a `match`.
Writing one there needs parentheses: `if (user { active: true }).active { … }`.

## Literals

The lexer hands over the text of a literal; the parser decodes it.

An integer literal decodes to a signed 64-bit value, and one too large to fit is an error.
A `-` before a number is part of that number, so the smallest whole number can be written; a `-`
before anything else is the prefix operator.
Blanks make no difference: `- 5` and `-5` are the same number, which is what lets the formatter
write the one canonical spelling of it without changing what the source says.
A postfix operator applies to the literal, so `-5.size` reads the field `size` of `-5`.
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
`an expression`, `a pattern`, `a function name`, `a trait`, `the end of the line`,
`an import, a type, a trait, an instance, a derive, or a function`, or the exact token,
as in `` `)` ``.
The found part names what is there the same way, or `the end of the file`.

Every parse error carries a code, and `docs/specs/diagnostics.md` is the catalogue of them.
An `expected <what>, found <what>` error is `L0100`, whatever it expected.

Nine failures are not about which token was found, and have their own words:

| Error               | Code    | Message                                    | Help                        |
|---------------------|---------|--------------------------------------------|-----------------------------|
| `fn` without a name | `L0100` | expected a function name, found `` `(` ``  | every function has a name   |
| unterminated string | `L0101` | this string has no closing quote           | add a closing `"`           |
| unknown character   | `L0102` | this character is not part of the language | (none)                      |
| number too large    | `L0103` | this number does not fit in a whole number | the largest whole number    |
| unknown escape      | `L0104` | `` `\q` `` is not an escape                | the escapes the language knows |
| chained comparison  | `L0105` | comparisons do not chain                   | compare twice, join with `&&` |
| nesting too deep    | `L0106` | this nests too deeply to parse             | how deep brackets may nest  |
| assigned to a value | `L0107` | only a name is assigned to                 | build the value it becomes  |
| partly named call   | `L0108` | this call names some of its arguments and not others | all of them or none |

An assignment names a name, which `docs/design.md` section 10 states.
`user.name = "Bob"` and `first(users).id = 1` are `L0107`, pointing at what was written there.
A value is changed by building the one it becomes rather than by reaching inside it.

`=` and `+=` are the whole of the rule, and the grammar above is what makes that so.
`++`, `--`, `-=`, `*=`, `/=`, and `%=` are not in it, so each is `L0100` where it is written:
the grammar expected an expression and the source wrote an operator.
`docs/design.md` section 2 says why they are refused rather than deferred, and
`docs/principles.md` question 9 is what a proposal for one has to answer.

## Executable examples

`tests/spec/parser/<name>.lm` files are parsed and compared with a sibling expectation file.
A `<name>.ast` file holds the parse tree, one node per line, `<indent><node> <start>..<end>`.
A `<name>.error` file holds `<start>..<end> <message>`, then a `help: <text>` line if there is one.
A `.lm` file has exactly one of the two, and `tests/siblings.lm` names the failing file.
An example is a whole program held to `docs/specs/executable-examples.md`, not a fragment.

The two files are the printed form of the parser's answer, and each line of it ends in `\n`.
Each level of depth indents a node by two spaces.
A node that only holds other nodes, such as `result` or `else`, has no span.
A string shows between quotes, and `"`, `\`, and each ASCII control character in it are escaped.
A newline, a tab, a return, and a zero are `\n`, `\t`, `\r`, and `\0`; another control is `\u{…}`.
Every other character is written as it is, so the printed form of a string has one spelling.

## The parser written in Bux

`compiler/parser.lm` is this parser, written in Bux.
It reads the tokens of `compiler/lexer.lm` by kind and builds the tree `compiler/ast.lm` declares.
It is the second phase of the Bux compiler, which `docs/implementation.md` section 6 states.

`parser.printed(source)` gives the printed form: the tree, or the error that stops the parse.
`ast.printed(program)` writes the tree, in the form of a `.ast` file.

The parser stops at the first error, and the error has a span, a message, and a help line.

`tests/parsing.lm` holds the parser to the properties below, on drawn text and drawn programs.
`tests/siblings.lm` holds it to every `.ast` and `.error` file under `tests/spec`, line for line.

## Properties

These hold and are checked by drawn properties in the runner:

1. Parsing never panics, on any input.
2. Parsing is deterministic.
3. An error's span is non-empty and lies within the source.
4. A generated well-formed program parses, and its item spans are ordered and non-overlapping.

The round trip `parse(print(ast)) == ast` needs a printer, and lands with it in Item 003.
