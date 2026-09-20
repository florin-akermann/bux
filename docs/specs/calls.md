# A call with its first argument in front

`maybe.or(fallback)` is the call `or(maybe, fallback)`, written with its first argument in front.

## Intent

`docs/design.md` section 11 states the form: the two spellings are one call, and the dot moves
nothing but the reader's eye.

Work reads in the order it happens.
`find_user(id).or(guest)` says what is looked for before what stands in for it, and
`or(find_user(id), guest)` says the same thing inside out.
A chain of three grows a bracket at each end when it is written plainly, so the reader meets the
last step first and counts brackets to find the first.

Nothing is declared to earn the form.
There is no method, no receiver type, and no dispatch: a function takes what it takes, and a
declared type has the form exactly as `Option` and `Int` do.
A language that puts methods on types has to say which types may have them and what happens when
two of them declare one name; a language where the form is only a spelling has neither question.

## The rule

The name after the dot is looked up in scope, exactly as a plain call looks its callee up.

```text
fn or(maybe: Option<T>, fallback: T) -> T {
    …
}

held := found.or(0)
```

`or` is the function in scope, `found` is passed first, and the rest of the arguments follow it in
the order they are written.
Which function is called is settled by the name alone, before any type is known, so nothing about
the receiver's type takes part in finding it.

The receiver is an argument and counts as one.
A call that passes the wrong number of them is `L0401` with the receiver counted, so
`found.or()` passes one argument to a function that takes two.

A function the module does not declare is `L0300` where the name is written, as a plain call of
the same name is.
Nothing about the form reaches further than a plain call reaches: the same names are in scope, and
the same ones are not.

## Which of the three a dot is

The name before the dot says what the dot does, and nothing else does.

| what is before it   | what the dot does        | example          |
| ------------------- | ------------------------ | ---------------- |
| a module            | reaches into the module  | `io.print("hi")` |
| anything else, with `(…)` | passes it first    | `maybe.or(0)`    |
| anything else, without `(…)` | reads a field   | `user.name`      |

A module is a name an import brings into scope, and a binding is never one, which
`docs/specs/modules.md` states.
So `io.print("hi")` is a function of the module `io` and `maybe.or(0)` is the function `or` in
scope, and no type is looked up to tell them apart.

A field and a call are told apart by the brackets.
`user.or` reads a field named `or` and is `L0402` where the type has no such field, and
`user.or(0)` is the call.
`user.name.length()` passes `user.name` first, because the dot before `length` follows a field
read and not a module.

## Naming the arguments

A call written this way names none of its arguments.

```text
fn rename(from: String, to: String) -> String {
    from + to
}
```

`old.rename(to: new)` is `L0423`: the receiver is an argument and there is no place to name it, so
the call would name some of its arguments and not others, which `docs/specs/arguments.md` refuses.
`old.rename(new)` is `L0409`, because `rename` gives two of its parameters one type and a call of
it says which is which.
`L0423` is raised before the arguments are counted, so `old.rename(from: old, to: new)` earns it
rather than `L0401`: the receiver is one of the arguments the count counts.
A call that has to name its arguments is therefore written plainly.
`rename(from: old, to: new)` is the whole of what is available to it.

A call whose parameters hold each other apart by type may be written either way, and `maybe.or(0)`
is one of those.

## The formatter

Canonical form keeps whichever spelling the author wrote.
The two are one call and not one form and its shorthand, so there is nothing for `lumen fmt` to
rewrite and nothing for it to prefer.
Spacing inside the call is the spacing of any other call, which `docs/specs/formatting.md` states.

## The errors

`L0423` is raised by inference, in `crates/types/src/error.rs`:

```text
error[L0423]: `rename` is written with its first argument in front, so this call names none of them
  --> demo.lm:2:5

  2 |     old.rename(to: new)
    |     ^^^^^^^^^^^^^^^^^^^

help: a call that names its arguments is written plainly, with every argument inside the brackets
```

The message names the function rather than the argument, because the answer is to rewrite the call
rather than to rename one value in it.

Every other refusal the form can earn is the refusal the plain call earns, raised where the plain
call earns it: `L0300` for a name nothing declares, `L0401` for a count, `L0400` for a type, and
`L0409` for a call that has to name its arguments.

## Properties

These hold and are checked with property-based tests:

1. `first.f(rest)` and `f(first, rest)` infer the same type wherever both are written.
2. The two spellings compile to the same instructions, apart from where they are written.
3. Canonical form leaves each spelling as the author wrote it.
