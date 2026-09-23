# Traits and instances

## Intent

A trait names what a type can do, and an instance is one type doing it.

`Eq` is the first: `==` compares two values only when their type has an instance of `Eq`, and a
type the JVM holds is no exception to that.
`docs/design.md` section 8 states the design; this spec states what is written, what is refused,
and what a constraint resolves to.

A constraint resolves while the program is compiled.
A generic is written once per set of types it is used at, which `docs/specs/codegen.md` states, so
the body written for a set of types calls the instances those types have.
Nothing is passed at run time, no method table is built, and no instance is looked up.

## Declaring a trait

```text
trait Eq<T> {
    fn is_equal(one: T, other: T) -> Bool
}
```

A trait declares a name, one type parameter, and one or more method signatures.
A signature is a function's first line and nothing else: a name, its parameters, and its result.
It has no body, because a trait says what a method is and an instance says what it does.

A signature writes the type of every parameter it takes.
A function may leave one out and let inference read it off the body; a signature has no body, so a
type left out of one is a type nothing would settle, and `L0419` refuses it where it is missing.
A method that gives nothing back writes no `->`, which is the one thing a signature may leave out.

One type parameter is what every trait the language has asks for.
`Eq<T>`, `Ord<T>`, `Show<T>`, and `IntegerLiteral<T>` each name one type and say what it can do.
A trait over two types is a question for the spec that needs one.

A trait's name is a name in the type scope, beside the types, because that is where it is written:
in `instance Eq<Point>` and in `T: Eq<T>`.
Each of its method names is a name in the value scope, beside the functions, because that is where
a method is called: `is_equal(one, other)`.
A trait is therefore refused where a type belongs, and a type is refused where a trait belongs.

A method's signature is written over the trait's type parameter, and it is the one type parameter
a signature may name.

## Giving a type an instance

```text
instance Eq<Point> {
    fn is_equal(one: Point, other: Point) -> Bool {
        one.across == other.across && one.down == other.down
    }
}
```

An instance names a trait, the type it is for, and a body for each of the trait's methods.

The type is written by name: `Eq<Point>` over a type that takes no arguments, and
`Eq<List<T>>` over one that does.

The name is a type, so a trait written there is refused with `L0311`, as one written where any
other type belongs is.

An instance writes a body for every method its trait declares, once, and for no other name.
A method it leaves out, a name the trait never declared, and a second body for one method are each
refused: an instance's method names are its trait's, so a name written twice would put no second
name in scope and one of the two bodies would be reached while the other was lost.

Each body's signature is the trait's, with the trait's type parameter standing for the instance's
type, so `is_equal` in `instance Eq<Point>` takes two `Point`s and gives back a `Bool`.
A body may leave a type out and let inference read it off the trait, so `fn is_equal(one, other)`
is the same declaration written shorter.
A body that writes a type the trait does not have is a mismatch, reported where it is written.

An instance declares no name.
Two instances each write `is_equal`, and neither of them is the `is_equal` anything calls: the trait
declares that name, and the instances say what it does at each type.

An instance is written in the module that declares the trait or in the module that declares the
type it is for.
The instances of a type travel with the type, so every module that reaches the type reaches them.
`docs/specs/modules.md` states what a module that imports the type reaches.

One trait and one type have one instance.
A second instance of one trait for one type is refused where it is written.

An instance is written by hand or by a derive, and the two are the same instance: what a derive
writes is what an author would have, and a second one either way is the second instance refused.
`docs/specs/derive.md` states which traits a type derives and what each derive writes.

### An instance for a type written with type parameters

```text
instance<T: Eq<T>> Eq<List<T>> {
    fn is_equal(one: List<T>, other: List<T>) -> Bool {
        ...
    }
}
```

A type that takes arguments gets one instance, and that one instance answers at every argument.
The instance declares its type parameters after the `instance` keyword, exactly as a function
declares its own, and it writes them as the arguments of the type it is for.
`instance<T: Eq<T>> Eq<List<T>>` is therefore the one `Eq<List<T>>` there is, and it answers for
`List<Int>`, for `List<String>`, and for `List<List<Int>>` alike.

Each type parameter takes a constraint where the body asks something of it, written exactly where a
function's constraint is written.
`is_equal` over two lists compares two elements with `==`, which resolves to `Eq<T>`, so the
instance declares `T: Eq<T>`.
A parameter the body asks nothing of takes none, so `instance<T> Show<Box<T>>` is written, and a
body that then asks something of `T` is refused with `L0418`, as a generic function's body is.

Every argument of the type is a type parameter the instance declares.
`instance Eq<Box<Int>>` writes a type rather than a parameter, so it would answer for `Box<Int>`
and leave `Box<Bool>` with none; name resolution refuses it with `L0318`.
An instance that writes more or fewer arguments than its type takes is refused with `L0401`, as a
type written with the wrong number of arguments is anywhere else.

One trait and one type have one instance, and the type here is `List` rather than `List<Int>`, so
`L0308` reads the name and nothing after it.
The methods are the trait's at the instance's type, as they are for any other instance: `is_equal`
takes two `List<T>` and gives back a `Bool`.

## The standard traits

`docs/design.md` section 8 names four traits every language wants and every type may opt into:
`Eq`, `Ord`, `Hash`, and `Show`.
Each is a trait like any other, each declares one method, and no type has any of them until it
writes the instance or derives it.

```text
trait Eq<T> {
    fn is_equal(one: T, other: T) -> Bool
}

trait Ord<T> {
    fn is_less(one: T, other: T) -> Bool
}

trait Hash<T> {
    fn hashed(value: T) -> Int
}

trait Show<T> {
    fn shown(value: T) -> String
}
```

`Hash` is a trait a type opts into rather than the `hashCode` every JVM object is born with, and
`Show` is one it opts into rather than `toString`.
`docs/design.md` section 2 keeps a value free of both: a type that never asked has no hash to be
put in a map by and no text to be printed as, and asking is writing the instance or the derive.

### What each one promises

`Eq` says when two values are one value, and it reads what they hold.
Two values are equal when everything they hold is equal, and nothing asks whether two references
are one object.

`Ord` says which of two values comes first.
`is_less` is the one method the four comparisons are written with, which
`docs/specs/operators.md` states.
The order is total: for any two values exactly one of `one < other`, `other < one`, and
`one == other` holds, so `Ord` and `Eq` never disagree.
It is also transitive: where `a < b` and `b < c`, `a < c`.
The law is a law about a type that has both; a type may have `Ord` alone, and `docs/specs/derive.md`
asks nothing of `Eq` when it writes one.

`Hash` gives a value a whole number that stands for what it holds.
Equal values hash alike, which is the whole of what a hash promises; two values that hash alike
may still differ, and nothing reads a hash as an answer about equality.
The law binds a type that has both, as `Ord`'s does.

`Show` renders a value as text a reader reads.
It is a function of what the value holds, so two equal values are shown alike.
A `String` is shown as the characters it holds and nothing more: `shown("ada")` is `ada`, because
`Show` is one rule for every type and a quote would be a rule for one of them.

### The instances the prelude has for them

The prelude has all four for `Bool`, `Int`, and `String`, and all four for `List<T>`:

```text
instance Eq<Bool>       instance Eq<Int>       instance Eq<String>
instance Ord<Bool>      instance Ord<Int>      instance Ord<String>
instance Hash<Bool>     instance Hash<Int>     instance Hash<String>
instance Show<Bool>     instance Show<Int>     instance Show<String>

instance<T: Eq<T>> Eq<List<T>>        instance<T: Ord<T>> Ord<List<T>>
instance<T: Hash<T>> Hash<List<T>>    instance<T: Show<T>> Show<List<T>>
```

All sixteen are source the prelude carries, a few lines each, and `library/prelude.lm` is where a
reader goes to find what one of them says.
`Hash<String>` reads a string by the characters it holds, over a `String.hashCode` declared with
the width `docs/specs/interop.md` states.

None of the sixteen has a body anything calls.
What one amounts to is written out where it is called — the JVM instruction for it, or a call of
the Java member behind it — which `docs/specs/library.md` states and `docs/specs/codegen.md`
writes out.

`false` comes before `true`, which is the order the two are written in and the order a JVM already
puts them in.
Two strings are ordered by their characters, one by one, and a string that begins another comes
first: `"a" < "ab" < "b"`.

`hashed` at `Int` is the whole number itself, at `Bool` is `0` for `false` and `1` for `true`, and
at `String` is a number worked out from the characters it holds.
Which number two unequal values work out to is not part of this spec, and nothing may be written
that depends on it.
`shown` at `Int` is the digits it is written with, at `Bool` is `true` or `false`, and at `String`
is the string.

Each of the four over `List<T>` walks the list and asks `T` for the instance of its own trait,
which is what the constraint on `T` promises.
Two lists are equal where they hold the same number of values and each value is equal to the one
beside it.
Two lists are ordered by their elements, one by one, and a list another begins comes first, which
is the order two strings are already in.
`hashed` at `List<T>` works one number out from the hash of every element, in order.
`shown` at `List<T>` is the elements shown, separated by `, `, between `[` and `]`, so
`shown([1, 2, 3])` is `[1, 2, 3]`.

`Eq`, `Ord`, `Hash`, and `Show`, and `is_equal`, `is_less`, `hashed`, and `shown`, are ordinary
prelude names rather than keywords, so a module declaring one of them is refused with `L0302`.

## Constraining a generic

```text
fn has_value<T: Eq<T>>(items: List<T>, value: T) -> Bool {
    for item in items {
        if is_equal(item, value) {
            return true
        }
    }
    false
}
```

A type parameter is written with the trait it is constrained by, which is what lets the body call
that trait's methods at it.

A type parameter takes one constraint for each trait its body asks of it.
Two of them are written with `+` between them: `K: Eq<K> + Hash<K>` is a key a body compares and
hashes, and it is the constraint every function of `map` and `set` writes over a key.
The order is the author's and nothing reorders it, because the constraints are one set of promises
however they are written down.
A body written over such a parameter calls the methods of either trait at it, and a use of the
generic is accepted only where what the parameter settled on has an instance of each of them.

A call of a trait method at a type parameter is accepted only where the parameter is constrained
by that method's trait.
A call of `has_value` is accepted only where what `T` settled on has an instance of `Eq`.
Neither is a run-time question: both are answered where they are written.

## How a constraint is resolved

A use of a trait method is resolved by the type its trait's parameter settled on:

- a named type reaches the one instance of that trait for that type;
- a type parameter of the function being compiled reaches the constraint that parameter declares;
- anything else is refused, because nothing names an instance.

A named type is read by its name alone, so `List<Int>` reaches the one instance of `Eq<List<T>>`
there is.
That instance asks its own constraint of each argument the type is written with, so `Eq` at
`List<Int>` is answered because `Eq` is answered at `Int`.
`Eq` at `List<Crate>` is answered only once `Crate` has an instance of `Eq`, and it is refused
naming `Crate` until then.

A type parameter answers what its constraint answers wherever it stands, and an argument of a type
written with arguments is one of those places.
So `Eq` at `List<T>` is answered inside a body written over `T: Eq<T>`, for the reason `Eq` at
`List<Int>` is answered.

A generic is compiled once per set of types, so the second case is a case only while type checking.
The body written for `has_value` at `Point` has `T` standing for `Point`, and the `is_equal` it
calls is the one `instance Eq<Point>` declares.
The instance `has_value` at `Int` resolves to is the one `library/prelude.lm` writes for `Int`,
and what that instance amounts to is written out in place rather than called.

`==` and `!=` are resolved the same way, against `Eq`.
A comparison of two values of a type with no instance of `Eq` is refused as it was before traits,
with `L0406`, and a comparison nothing settled is a comparison of `Int`s, which is the one default
the language keeps.

## What the library writes and what the compiler supplies

`library/prelude.lm` is Lumen source the compiler carries, which `docs/specs/library.md` states,
and it declares every trait and writes every instance of one for `Bool`, `Int`, `String`, and
`List<T>`.
The four standard traits are written out above; `docs/specs/operators.md` writes out the six the
arithmetic operators are, and `docs/specs/literals.md` writes out `IntegerLiteral`.

`todo` is the one function the compiler supplies, because a hole has no body for the library to
write.
The prelude's instances over `List<T>` read a list with `at`, which is `list`'s name and the
compiler's function, and `docs/specs/library.md` states that the prelude reaches it too.
`Bool`, `Int`, `String`, and `List` stay the compiler's as well, because what they are made of is
the JVM rather than a declaration, which `docs/specs/library.md` states.
No instance, trait, or function of the prelude other than `todo` is the compiler's.

A module writing `instance Eq<Int>` is refused with `L0308`, because there already is one, and a
module declaring its own `Eq` is refused with `L0302`, exactly as one declaring its own `todo` is.
Which of the two wrote the one already there makes no difference to either refusal.

No instance over `Bool`, `Int`, or `String` has a body anything calls.
`is_equal` at `Int`, at `Bool`, or at `String` is written out where it is called, as `or` is, and
the comparison it writes is the one `==` already wrote: two whole numbers or two truth values as the
JVM compares them, and two strings by the characters they hold.
The body in `library/prelude.lm` is what says in Lumen what that instruction does, and reading it
is what holds it to its type.
Each instance over `List<T>` is different: its body in `library/prelude.lm` is the one lowered.
`docs/specs/codegen.md` states the class that the method is written into.
Nothing asks whether two references are one object, which `docs/specs/codegen.md` requires.

## What is written

An instance's method is a static method of the module class, named for its trait, its type, and
itself, joined by `$`: `Eq$Point$is_equal`.
`$` is legal in a JVM method name and Lumen writes no operator with it, which is the same reason
`docs/specs/codegen.md` names a specialized generic that way.

An instance's method is written whether anything calls it or not, as a function that declares no
type parameter is, because it names no type parameter of its own.

An instance that declares type parameters is generic in them, exactly as a function declaring them
is, so its method is written once per set of types it is used at and not at all until one is.
The name is the plain one, and then the arguments the type is written with, in that order and each
behind a `$`: `Eq$Box$is_equal$Int` is `instance<T: Eq<T>> Eq<Box<T>>` at `Int`.
It is the type's own order rather than the instance's, because the name is read off the type and
nothing else, which is what has a module reach another module's instance without reading its tree.
An argument that itself takes arguments is written out whole, so `Eq<Box<List<Int>>>` reaches
`Eq$Box$is_equal$List$Int`, and the argument is what keeps two uses of one instance apart.

`List` is the compiler's type, so no module declares it, and the prelude declares its instances.
The four that the prelude writes over `List<T>` are methods of the class of the prelude.
`Eq$List$is_equal$Int` is one method of that class, whichever module uses it.

A call of a trait method is an `invokestatic` of the method the instance wrote, or, over a type
the JVM holds, the instruction that instance amounts to written out in place.
`one == other` over a type with an instance is that same call, and `one != other` is that call
with the answer flipped.

## Canonical form

A trait is written with one signature per line and no blank line between two of them, as a record
type is written with one field per line.

An instance is written with one function per line and one blank line between two of them, as a
module is written with one blank line between two items.

A trait and an instance each take the indentation and the brace placement every block takes, which
`docs/specs/formatting.md` states.

A trait's name and the type an instance is for are written in `PascalCase`, and a method's name in
`snake_case`, which `docs/specs/naming.md` already asks of a type and a function.

## The errors

| Code | Raised when |
| --- | --- |
| `L0302` | A module declares a prelude trait or method, such as `Eq` or `is_equal`. |
| `L0308` | A trait already has an instance for the type a second instance names. |
| `L0309` | An instance leaves a method out, writes an undeclared one, or writes one twice. |
| `L0310` | Something that is not a trait is written where a trait belongs. |
| `L0311` | A trait is written where a type belongs. |
| `L0318` | An instance writes an argument of its type that is no type parameter it declares. |
| `L0400` | An instance's method has a signature the trait's method does not. |
| `L0401` | An instance writes more or fewer arguments than the type it is for takes. |
| `L0406` | An operator is written over a type that has no instance of the trait it is. |
| `L0418` | A trait method is used at a type with no instance of its trait. |
| `L0419` | A parameter of a method a trait declares states no type. |

`L0308`, `L0309`, `L0310`, `L0311`, and `L0318` are raised by name resolution, which
`compiler/resolver.lm` words.
`L0401`, `L0418`, and `L0419` are raised by type inference, which `compiler/refusal.lm`
words.

## Properties

These hold and are checked by drawn properties in the runner:

1. A trait, an instance over a plain type, an instance over type parameters, and a constraint
   each survive printing and parsing unchanged.
2. Canonical form is idempotent over a file holding a trait and an instance.
3. A trait method called at a type with an instance resolves to that instance and to no other.
4. A generic constrained by a trait is written once per type it is used at, and each body calls
   the instance of that type.
5. A program with two instances of one trait for one type is refused, whatever order they are in.
6. `==` is accepted over exactly the types that have an instance of `Eq`.
7. `is_less`, `hashed`, and `shown` are each accepted over exactly the types that have an
   instance of the trait declaring them, as `==` is.

What each instance over those three types answers is a claim about a running program, so it is
held to by `tests/spec/traits/supplied.lm` rather than by a property: that `is_less` is a total
order over `Bool`, `Int`, and `String` and is transitive, that equal values hash alike, and that
`shown` renders each of the three as this spec states.
What the four over `List<T>` answer is held to the same way, by
`tests/spec/traits/over_a_list.lm`, and what a module's own instance over its own type answers by
`tests/spec/traits/over_a_generic_type.lm`.
