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
map is made of are writable as well: `map.Empty`, `map.Leaf`, and `map.Branch`.
A map written out of those by hand is a map, and `get` reads it exactly as the trie below says it
does: it is read by the hash of the key it is asked for, and not by the order anything was
written in.
A program builds a map by `empty` and `insert`, because those two are what put an entry where the
hash of its key says it goes.

## The shape

Each of the two is a hash array mapped trie, which is the textbook persistent hash table.

A map is a value, so `insert` gives back a new map and the map it was handed stays readable.
A table written over could not keep the old map intact, and no array crosses an `extern` boundary
for such a table to be held in, so a table is not a shape this language can hold at all.
A trie answers both: a node that changes is a node copied with one child put in place of another,
and every map that shares the rest of the trie still holds everything it held.

A node holds up to thirty-two children, indexed by five bits of the key's hash.
Which five bits is what the walk has left of that hash: the root reads the lowest five, the node
below it reads the next five, and each node below that reads the five after those.
A leaf holds one entry, and a node whose five bits are the same for two keys is pushed one level
down, where the next five bits tell the two apart.

Two keys whose whole hashes are equal are told apart by no five bits of them at all.
Those two share a collision leaf, which holds a list of entries and is walked by `Eq<K>`.
`Hash<K>` is a function a program writes, and a program may write one badly, so a collision leaf
is the ordinary case of the shape rather than a case to be surprised by.

A hash is sixty-four bits and a walk reads five of them at a time, so a walk is at most thirteen
nodes deep whatever a program puts in.
A walk carries what is left of the hash rather than counting how far down it has gone, so a leaf
holds what was left of the hash where the leaf stands.
The bits are read from the lowest up, and a negative hash reads as the whole number it is, so any
two hashes that differ at all part at one of those thirteen nodes.

`Set<T>` is the same trie: `set` is written over `map`, and a set holds a key for each value it
was given and one value at every one of those keys.
The walk of the trie is therefore written once rather than twice, which is what one name having
one definition asks for.

## The cost, stated plainly

`get`, `insert`, and `has_value` each walk the trie to the depth the key's hash takes them, which
is the logarithm to base thirty-two of the entries the map holds.
`insert` copies one node of up to thirty-two children at each level of that walk, because a list
is read at an index and grown at the end and nothing writes into one.
A node changed is therefore a `for` loop that pushes every child of it onto a fresh list, which
costs the width of that node.

That is the cost, and it is not constant time.
It is the best cost known for a table that is a value, which is the one thing this language can
hold: a table that answers in constant time answers by writing over the slot it was asked about,
and a map that did that would take the old map's entry away with it.
What a program pays to fill one of these is the entries it puts in, each at that cost, and not the
square of them that a list of entries cost before.

Nothing here is tuned.
There is no transient, no in-place path for a map nothing else holds, and no bitmap-compressed
node.
A node is thirty-two slots written over `List`, and an empty slot holds the empty map.

The alternative weighed was the compressed node the textbook goes on to: a bitmap of which slots
are used, beside a list holding only those.
It holds a small map in less space and copies fewer children per level, and it costs the same
walk: the depth is the same, and the width of a node is a constant either way.
It also asks for the bit counting that turns a bitmap and a slot into an index, which is arithmetic
this library would write and read for no change in what anything costs.
A library data structure here has the best known asymptotic cost, written plainly and never tuned,
so the plain node is what is written and the measurement on a real program is what would change it.

## What a key must be

A key is a type a program compares and hashes, so `K` is constrained by `Eq<K>` and by `Hash<K>`.

The walk reads the hash, and the leaf it ends on is walked by equality, so both constraints are
read by a body here rather than promised for nothing.
The two are written with `+` between them, which `docs/specs/traits.md` states, and every function
of `map` and of `set` that reads or writes a key writes both.
A key at a type with no instance of either is refused where the call is written, with `L0418`, as
a missing instance is refused anywhere.

A constraint on a library generic is answered by the instance the key's own type has, which
`docs/specs/codegen.md` states the specialised method calls by name.
A key is therefore a `Bool`, an `Int`, or a `String`, and a type the program declares both
instances for, by hand or by `derive Eq, Hash for Kept`.
That type can be declared in another module, because the instances of a type travel with it.
The prelude writes `Hash<List<T>>` beside `Eq<List<T>>`, which `docs/specs/traits.md` states, so a
`List<T>` is a key wherever `T` is one.

`Hash` promises only that equal values hash alike, which `docs/specs/traits.md` states.
Two keys that are not equal may hash alike, and the collision leaf is where they both land; what
the map answers is the same either way, and only what the walk costs changes.

## The functions

Each cost below is in entries the map holds, or values the set holds.

```text
map:
fn empty<K, V>() -> Map<K, V>
fn insert<K: Eq<K> + Hash<K>, V>(into: Map<K, V>, key: K, value: V) -> Map<K, V>
fn get<K: Eq<K> + Hash<K>, V>(within: Map<K, V>, key: K) -> Option<V>
fn merged<K: Eq<K>, V>(node: Map<K, V>, rest: Int, key: K, value: V) -> Map<K, V>
fn pushed<K, V>(node: Map<K, V>, rest: Int, key: K, value: V) -> Map<K, V>
fn with_child<K, V>(children: List<Map<K, V>>, slot: Int, child: Map<K, V>) -> List<Map<K, V>>
fn children_of<K, V>(within: Map<K, V>) -> List<Map<K, V>>
fn no_children<K, V>() -> List<Map<K, V>>
fn replaced<K: Eq<K>, V>(entries: List<Entry<K, V>>, key: K, value: V) -> List<Entry<K, V>>
fn value_among<K: Eq<K>, V>(entries: List<Entry<K, V>>, key: K) -> Option<V>
fn is_at_key<K: Eq<K>, V>(entry: Entry<K, V>, key: K) -> Bool
fn value_of<K, V>(entry: Entry<K, V>) -> V
fn slot_of<K, V>(step: Step<K, V>) -> Int
fn branch_of<K, V>(step: Step<K, V>) -> Map<K, V>
fn child_at<K, V>(within: Map<K, V>, slot: Int) -> Map<K, V>
fn without_digit(code: Int) -> Int
fn digit_of(code: Int) -> Int
fn has_children<K, V>(within: Map<K, V>) -> Bool
fn hash_within<K, V>(within: Map<K, V>, fallback: Int) -> Int
fn entries_of<K, V>(within: Map<K, V>) -> List<Entry<K, V>>

set:
fn empty<T>() -> Set<T>
fn insert<T: Eq<T> + Hash<T>>(into: Set<T>, value: T) -> Set<T>
fn has_value<T: Eq<T> + Hash<T>>(within: Set<T>, value: T) -> Bool
fn held_by<T>(within: Set<T>) -> map.Map<T, Bool>
```

The first three of each are what a program writes; the rest are the steps of the walk.
`map` declares three types beside them: `Map<K, V>` itself, `Entry<K, V>`, which is one key and
the value at it, and `Step<K, V>`, which is one branch the walk went through and the slot it took.
`set` declares `Set<T>`, which is one `map.Map<T, Bool>`.

`empty` is one step in either module.
`map.get` and `set.has_value` each walk the trie by the hash of the key they are handed, and then
walk the leaf they end on by equality.
`map.insert` walks the same way, and then writes the node it ended on again and every node above
it, which is what gives back a map without taking anything away from the one it was handed.
`set.insert` is `map.insert` at the value `true`, so a value put in twice is one key written
twice and the set that already held it.

Every walk is a `for` loop over a `var`, and no function here calls itself.
A walk written as a function that calls itself runs out of stack on a map a program can build,
and running out of stack is a runtime failure a program can reach, which `docs/design.md`
section 2 says there is none of.
The steps of each walk are the functions below the three above, because a `match` arm is an
expression and an expression cannot assign; they are public because every top-level name is.

`get` gives an `Option<V>`, because a key the map holds no entry for is a case the type has to
say rather than a case a program finds out by running.
Nothing here is partial, and nothing here panics.

`insert` puts one value at one key: a key already there keeps its place in its leaf and gives up
its value.
`set.insert` holds a value once however often it is given.
What the map costs is what it holds, and not what was ever put into it.

## What the language writes it with

Everything here is written in plain Bux, in `library/map.lm` and `library/set.lm`.
The whole of it is `List`, `list.at`, `list.push`, declared types, `match`, `if`, `var`, and `for`
loops, which is what `docs/specs/library.md` holds every library function to.

A node is a `List<Map<K, V>>` of thirty-two children, and nothing writes into a list.
So `with_child` is a `for` loop that pushes every child onto a fresh list, putting the new child
in where the slot it is given comes round.
That loop costs the width of the node, which is the thirty-two children the copy above is.
`list.push` costs what the list holds today, which `docs/specs/library.md` states, so writing a
node again costs the square of its width rather than its width.
A node is thirty-two children wide whatever the map holds, so that square is a fixed cost that
what the map holds never grows.

The five bits a node indexes by are read with `%` and `/` rather than with a bit operator, which
the language does not write.
`digit_of` gives what is left of a hash modulo thirty-two, counted up from zero where the hash is
negative, and `without_digit` takes those five bits off, rounding down so that the sign is kept.
The two together are the base-thirty-two reading of a whole number, which is what makes two
different hashes part within thirteen nodes.

## Values, and equality

Neither type has identity: nothing can ask whether two maps are one object, which
`docs/design.md` section 9 states of every value.

Keys are in no order a program may read, and the order they went in is no part of what a map is.

Neither type has an `Eq` instance.
Two maps holding the same entries are the same map to every program that reads them, and an `Eq`
that answered so would have to read the whole trie of each of them.
That is a walk a map has no function for, because a map has no iterator.
So `==` over a map is refused as it is over any type with no instance, and a program compares what
it reads out of a map rather than the map itself.

## The errors

| code    | what it refuses                                                                 |
| ------- | ------------------------------------------------------------------------------- |
| `L0418` | a key, or a member, at a type this module reaches no `Eq` or `Hash` instance for |
| `L0406` | `==` over a `Map` or a `Set`, which have no instance of `Eq`                     |

Neither code is new, and neither is about collections: a constraint with no instance and a type
with no instance are refused here exactly as they are refused anywhere.

## Properties

These hold and are checked with property-based tests:

1. `get` after `insert` gives back the value that went in, at whatever key it went in at.
2. `insert` at a key already there replaces its value and leaves every other key readable.
3. `has_value` is true of every value inserted into a set and false of every value that was not.
4. The three above hold over hundreds of keys, keys whose hashes are equal among them.

The library is Bux source, so what it does is observable only by running a program that uses it.
Each case of each property writes one program, compiles it with the compiler under test, runs it
on a JVM, and compares what that program wrote with what the drawn keys say it had to write.
A run needs a JDK, which no test may need, so a case is skipped with a named reason where
`JAVA_HOME` names none.
Property 4 draws hundreds of keys written as lists, two of which hash alike for each whole number
it draws, so a leaf holding more than one entry is what most of its cases read.
