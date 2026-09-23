# Arguments

A call passes its arguments in order, and where the order is the only thing holding them apart,
the call writes the parameter names too.

## Intent

`rename(a, b)` and `rename(b, a)` both compile when `a` and `b` are both `String`s, and one of
them is wrong.
The type system has nothing to say, because the types agree; the mistake shows up when the program
runs, or never.

mycs lints a swappable pair of arguments after the fact.
A structural mistake a type system can make unwriteable belongs in the language rather than in a
linter, so Lumen refuses the call instead.

## The rule

A call names all of its arguments or none of them.

```text
rename(from: old, to: new)
```

A name is written before its value, joined by `:`, exactly as a record writes a field.

The names are written in the order the declaration lists the parameters.
Naming does not reorder anything: the call still passes its arguments in order, and the names say
which order that is.
A swapped pair is then `L0410` where it is written rather than a wrong answer where it is read.

A call must name its arguments when the declaration gives two of its parameters one type.
Nothing else tells those two apart, so the call says which is which or does not compile.

```text
fn rename(from: String, to: String) -> String {
    from + to
}
```

`rename(old, new)` is `L0409`, and `rename(from: old, to: new)` is how it is written.

A call whose parameters all have different types may be written either way.
The types already hold the arguments apart, so naming them is the author's to choose.

## One way to write a call

A call writes the name of what it calls, then every argument inside the parentheses.
There is no second spelling: a dot never moves an argument in front of the name.

A dot does one of two things, and the name before it says which.
After a module, it reaches a name inside the module, as `io.println("hi")` does.
After a value, it reads a field, as `user.name` does.
A binding is never a module, which `docs/specs/modules.md` states, so no type is looked up to tell
the two apart.

`maybe.or(0)` is therefore `L0433`, and `or(maybe, 0)` is how it is written.
`user.name(0)` is `L0433` too, because a dot after a value reads a field and calls nothing.
The name after that dot is not looked up, so `count.missing(2)` is `L0433` and not `L0300`.

The `help:` of `L0433` is the plain call, written from the source with the value first.
Where the call names its arguments and this module declares the function at the top level, the
value is named for the first parameter: `old.rename(to: new)` helps with
`rename(from: old, to: new)`.
Anything else has no parameter names to write, so its help passes every argument in order.

## Which types count as one

The types compared are the ones inference settled for the declaration, so a signature the author
left unwritten counts exactly as one they wrote out.

A type parameter counts as a type: `fn pair<T>(first: T, second: T)` gives two parameters one
type, and a call of it names them.
`fn apply<T, U>(value: T, other: U)` does not, however a call happens to instantiate `T` and `U`.
The rule is about the signature, so it reads the same at every call site.

## A function declared above its caller

A call may reach a function declared above it, which mutual recursion always does.

```text
fn is_even(steps: Int, label: String) -> Bool {
    if steps == 0 {
        true
    } else {
        is_odd(steps - 1, label)
    }
}

fn is_odd(steps: Int, label: String) -> Bool {
    if steps == 0 {
        false
    } else {
        is_even(steps - 1, label)
    }
}
```

The types of `steps` and `label` differ, so both calls pass their arguments in order and compile.
Where the two functions sit is not part of the rule.
A call of a function above it reads the same signature as a call of a function below it.
That holds for a signature the author left unwritten too.
`fn is_even(steps, label)` counts the types that inference settled for its parameters.
Nothing about the rule rests on the order in which the compiler walks the bodies of a module.
`tests/spec/arguments/mutual_recursion.lm` is the example.

## What has no names to write

A constructor carries its values in order and has no names to write for them.

```text
type Span = Span(Int, Int)
```

`Span(0, 10)` is a call of a constructor, not of a function, so it is positional however its types
run.
`Span(len: 10, start: 0)` is `L0411`: the names look as though they say where each value lands,
and a constructor has nothing to check them against, so they would read as a promise nothing keeps.
A variant that wants its values named declares them as fields, which a record variant does, and
then it is written `Span { start: 0, len: 10 }` and the names are the fields'.

The rule is about the functions a module declares.
Version 0.1's prelude declares `or` and `todo`, which take parameters of different types, so no
call of either is ever asked to name one.
Neither is declared here, so neither has parameter names to hold a call to, and naming the
arguments of one is `L0411` as well.

A name reached through a module is the third, and it is one for the same reason as the prelude.
What a module offers is the type of each function and of each constructor it declares.
A type holds no parameter name, so `demo.hello(name: "world")` is `L0411`.
A constructor of another module is no different, and `demo.Sent(how: "post")` is refused too.
Each is written with its values in order.

## The errors

`L0409` is a call that must name its arguments and does not:

```text
error[L0409]: `rename` gives two parameters the type `String`, so this call names its arguments
  --> demo.lm:2:5

  2 |     rename(old, new)
    |     ^^^^^^^^^^^^^^^^

help: a call names its arguments when the declaration gives two parameters one type
```

`L0410` is an argument named something other than the parameter it is passed for:

```text
error[L0410]: this argument is named `to`, and the parameter here is `from`
  --> demo.lm:2:12

  2 |     rename(to: new, from: old)
    |            ^^^^^^^

help: arguments are named in the order the declaration lists its parameters
```

`L0411` is a call that names arguments where what it calls has no names to hold them to:

```text
error[L0411]: `Span` is a constructor, so it carries its values in order and names none
  --> demo.lm:2:5

  2 |     Span(len: 10, start: 0)
    |     ^^^^^^^^^^^^^^^^^^^^^^^

help: a variant whose values want names declares them as fields and is built as a record
```

`L0433` is a call written with a value in front of the name:

```text
error[L0433]: a dot after a value reads a field, so this call of `or` is written plainly
  --> demo.lm:2:5

  2 |     maybe.or(0)
    |     ^^^^^^^^^^^

help: write `or(maybe, 0)`
```

All four are raised in `compiler/infer.lm`.
`L0433` is raised as inference reaches the call, before anything else about it is counted or met.
`L0410` and `L0411` are raised as inference reaches the call.
`L0409` is raised once inference has walked every function of the module.
A function declared above its caller is walked after the caller, so its types settle later.

Two refusals come before `L0409`, `L0410`, and `L0411`, because each settles what those three use.
A call with the wrong number of arguments is `L0401`: how many there are is settled before which
of them is which.
An argument of the wrong type is `L0400`: what the arguments are is settled before whether the
call has to name them.
A swapped pair of one type is unaffected by that order, because a pair of one type never clashes,
which is the whole reason the rule exists.

A call that names some of its arguments and not others is neither of these: it is `L0108`, and the
parser refuses it, because the two forms are two shapes of the grammar rather than one.

```text
error[L0108]: this call names some of its arguments and not others
  --> demo.lm:2:23

  2 |     rename(from: old, new)
    |                       ^^^

help: a call names all of its arguments or none of them
```

## A parameter is never a bare `Bool`

`open(true)` says nothing.
The reader has to find the declaration to learn what is true, and `open(false)` is one keystroke
away from a program that does the other thing without looking wrong.

```text
fn open(path: String, read_only: Bool) -> File {
```

That parameter is `L0412`.
A two-variant type takes its place, and the call then says which of the two it means:

```text
type Mode =
    | ReadOnly
    | ReadWrite

fn open(path: String, mode: Mode) -> File {
```

`open(path, ReadOnly)` reads where `open(path, true)` did not, and a third mode is a variant
rather than a second flag.
`docs/specs/exhaustiveness.md` then makes every `match` on it answer for the new one.

The rule is the same shape as the naming rule above: a mistake a type system can make unwriteable
belongs in the language rather than in a linter, and mycs lints `Flag Argument` after the fact.

### The one carve-out

A function whose parameters are all `Bool` and whose result is `Bool` is a boolean operation, and
its parameters stay writable.

```text
fn implies(first: Bool, second: Bool) -> Bool {
    !first || second
}
```

`Bool` is what such a function is about, rather than something it is told.
Nothing else is carved out: `fn spoken(loudly: Bool) -> String` takes a flag however its parameter
is named, because what comes back is not a `Bool` and so the `Bool` was a choice, not an operand.

The types read are the ones inference settled, as the naming rule reads them, so a parameter the
author left untyped is held to whatever type it turned out to have.
A type parameter is not a `Bool`: `fn pick<T>(value: T)` is untouched however a call instantiates
it, because the rule is about the signature.
A parameter the body never constrains turned out to be a type parameter rather than a `Bool`,
so a call of it passes `true` as freely as it passes anything else.

A record field, a variant payload, a binding, and a result type may each be `Bool`.
The rule is about what a call passes, which is the one place a bare `true` loses its meaning.

### A signature an author did not choose

The rule asks an author to have declared a two-variant type instead, so it reaches only the
signatures they wrote.
An instance method's is not one of them.

```text
trait Hash<T> {
    fn hashed(value: T) -> Int
}

instance Hash<Bool> {
    fn hashed(value: Bool) -> Int {
```

`Bool` is there because the trait wrote `T` and the instance settled it, and neither of those is a
choice the method made.
There is no flag its author could have declined to write, and the two-variant type the rule asks
for is one the trait would have to have taken.

So the rule reaches an instance method through its trait rather than at the instance, which
`docs/design.md` section 11 states.
A trait's own signature is held to it, at the parameter the trait wrote:

```text
trait Hash<T> {
    fn hashed(value: Bool) -> Int
}
```

That is `L0412` where the trait writes it, because every instance of the trait would have to take
the flag and none of them could decline it.
A trait method whose parameters and result are all `Bool` keeps them, exactly as a function does.

Nothing is given up by reading the trait instead of the instance: a bare `Bool` in an instance
method is one the trait wrote or one the instance settled a type parameter on, and the second is
the type the instance is for rather than a parameter anything passes.
Everything else a function's body is held to, an instance method's body is held to.
`docs/specs/library.md` is where it comes up, because `instance Hash<Bool>` is one the prelude
writes.

`L0412` is the refusal:

```text
error[L0412]: this parameter is a `Bool`, so a call of `open` passes `true` and says no more
  --> demo.lm:1:23

  1 | fn open(path: String, read_only: Bool) -> File {
    |                       ^^^^^^^^^^^^^^^

help: declare a two-variant type and take that instead, so the call says which of the two
```

It is raised where the parameter is written, because the declaration is what changes.
It is reached after the body, because the type it reads is the one inference settled.
A body that does not typecheck is `L0400` first, as it is before every rule here.

## Properties

These hold and are checked by drawn properties in the runner:

1. A call of a declaration that repeats a type compiles when it names its arguments, and does not
   when it passes them positionally.
2. A call of a declaration whose parameter types all differ compiles either way.
3. A call that compiles passing its arguments in order still compiles naming them in that order.
4. A function with a `Bool` parameter compiles when every parameter and the result is `Bool`, and
   does not when anything else about the signature differs.
