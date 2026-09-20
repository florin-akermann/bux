# Collections

`Map<K, V>` and `Set<T>` are library types, written in Bux over what the language already gives.

## Intent

`docs/implementation.md` section 4 names collections among the areas the standard library covers,
and version 0.1 has only `List`.
A program that wants to answer "what is at this key?" writes the loop itself, and every program
that wants it writes the same loop.
That is the test `docs/specs/library.md` states, and a map passes it.

`docs/design.md` section 9 states what the two types are.
This spec states where they live, what each function is, what each costs, and what a key must be.

The library is small, and these two types are smaller still.
A map is built by `empty` and `insert` and read by `get`, and that is all it does.
A map has no iterator, which `docs/implementation.md` section 4 states outright, and neither has a
length, a removal, nor a literal: each waits for a program that cannot be written without it.

## The modules

`map` and `set` are library modules, carried by the compiler as `list` and `strings` are.
Neither is a prelude name, so a program reaches one by importing it:

```text
import map

fn oldest() -> Int {
    ages := map.insert(map.insert(map.empty(), "ada", 36), "alan", 41)
    or(map.get(ages, "ada"), 0)
}
```

One module holds one type, because one name has one definition.
`empty` is the map's in `map` and the set's in `set`, and a single module holding both could
declare only one of them.

The type is reached through the module too: `map.Map<String, Int>` is what `ages` above is, and
`set.Set<Int>` is what a set of whole numbers is.

Every name a module declares is public, which `docs/specs/modules.md` states, so the variants a
map is made of are writable as well: `map.Empty` and `map.Entry`.
A map built out of those rather than out of `insert` is read back exactly as it was written, and
`get` gives the value of the first entry whose key is equal.

## What a key must be

A key is a type equality is written over, so `K` is constrained by `Eq<K>`.

`get` finds the entry whose key is equal to the one it was handed, and equality is the only thing
it asks of a key.
Hashing asks for a table to bucket into, a table is an array, and an array is a JVM type the
language cannot yet name; `docs/implementation.md` section 10's `extern` declaration is what names
one.
So `Hash<K>` would be a constraint no body here reads, and a constraint nothing reads is a promise
a caller keeps for nothing.
It joins `Eq<K>` in the version that has the table, and not before.

A constraint on a library generic is answered by an instance the library itself reaches, which
`docs/specs/modules.md` states are the prelude's.
A key is therefore a `Bool`, an `Int`, or a `String` in version 0.1, and a type the program itself
declares is refused as `L0424` however the program came by its own `Eq`.

## The functions

Each cost is in entries the map holds, or values the set holds.

```text
map:
fn empty<K, V>() -> Map<K, V>
fn insert<K: Eq<K>, V>(into: Map<K, V>, key: K, value: V) -> Map<K, V>
fn get<K: Eq<K>, V>(within: Map<K, V>, key: K) -> Option<V>
fn is_at_another_key<K: Eq<K>, V>(within: Map<K, V>, key: K) -> Bool
fn is_an_entry<K, V>(within: Map<K, V>) -> Bool
fn value_at<K, V>(within: Map<K, V>) -> Option<V>
fn behind<K, V>(within: Map<K, V>) -> Map<K, V>
fn moved<K, V>(from: Map<K, V>, onto: Map<K, V>) -> Map<K, V>

set:
fn empty<T>() -> Set<T>
fn insert<T: Eq<T>>(into: Set<T>, value: T) -> Set<T>
fn has_value<T: Eq<T>>(within: Set<T>, value: T) -> Bool
fn is_at_another_value<T: Eq<T>>(within: Set<T>, value: T) -> Bool
fn is_a_value<T>(within: Set<T>) -> Bool
fn behind<T>(within: Set<T>) -> Set<T>
```

The first three of each are what a program writes; the rest are the steps of the walks.

`empty` is one step in either module.
`map.get` and `set.has_value` each walk until they find what they were handed, and to the end when
it is not there.
`map.insert` walks to the key it replaces, and to the end when the key is new; `set.insert` reads
the set first, so it walks to the value it already holds and to the end when it does not.
Building a map of a thousand keys therefore costs a thousand walks, and a set the same: what a
program pays to fill one of these is the square of what it puts in.

Every walk is a `for` loop over a `var`, and no function here calls itself.
A walk written as a function that calls itself runs out of stack on a map a program can build,
and running out of stack is a runtime failure a program can reach, which `docs/design.md`
section 2 says there is none of.
The steps of each walk are the functions below the three above, because a `match` arm is an
expression and an expression cannot assign; they are public because every top-level name is.

`get` gives an `Option<V>`, because a key the map holds no entry for is a case the type has to
say rather than a case a program finds out by running.
Nothing here is partial, and nothing here panics.

`insert` puts one value at one key: a key already there keeps its place and gives up its value.
`set.insert` holds a value once however often it is given, so inserting one twice is the set that
already held it.
Writing the entry in front of the old one would read back the same and cost nothing to write, and
it is not what either does: a key written a thousand times would leave a thousand entries behind,
and every later read would walk past all of them.
What the map costs is what it holds, and not what was ever put into it.

The cost is what the shape buys: a map is its entries one after another, and a walk of them is
what every function here is.
A table would make `get` one step and needs an array; until then the cost is written down rather
than hidden.
These two types are for tens and hundreds of entries.
A program holding thousands pays a square to fill one, and is better off writing the shape its
own problem wants.

## Values, and equality

Neither type has identity: nothing can ask whether two maps are one object, which
`docs/design.md` section 9 states of every value.

Neither has an `Eq` instance either.
Two maps holding the same entries are the same map to every program that reads them, and they are
not the same value: `insert` keeps each key where it was first written, so the order the entries
went in survives in what the map is made of.
An `Eq` that answered the first question would have to stop the order mattering first, and that
asks for keys that can be put in an order, which is a second constraint on `K` the language does
not write.
So `==` over a map is refused as it is over any type with no instance, and a program compares what
it reads out rather than the map itself.

## The errors

| code    | what it refuses                                                           |
| ------- | ------------------------------------------------------------------------- |
| `L0424` | a key, or a member, at a type the prelude reaches no `Eq` instance for     |
| `L0406` | `==` over a `Map` or a `Set`, which have no instance of `Eq`               |

Neither code is new, and neither is about collections: a library generic and a type with no
instance are refused here exactly as they are refused anywhere.

## Properties

These hold and are checked with property-based tests:

1. `get` after `insert` gives back the value that went in, at whatever key it went in at.
2. `insert` at a key already there replaces its value and leaves every other key readable.
3. `has_value` is true of every value inserted into a set and false of every value that was not.
