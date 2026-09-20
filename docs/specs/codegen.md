# Code generation

## Intent

A checked module becomes class files a JVM can load.
The phase consumes the typed tree, lowers it to a JVM intermediate representation, and writes the
class files that representation describes.
`docs/implementation.md` section 1 sets the target: JDK 28 or later, and no older JVM.
Every class written is a Valhalla value class, which is how a Lumen value stays a value on a JVM.

Nothing in this spec is visible from Lumen.
A reader of a Lumen program never needs it, and no diagnostic mentions a class, a descriptor, or a
stack frame.
It is written down because the output is a contract with the JVM, and because two compilations of
one source must produce the same bytes.

## What is written

`lumen build <file>` writes class files beside the source file.
A module is one source file, so `demo.lm` yields these:

```text
demo.class                 the module: one static method per function
demo/User.class            one class per type the module declares
demo/Payment$Failed.class  one class per variant of an algebraic data type
lumen/Option.class         the prelude types, which every module may reach
```

The module class is named after the file, so a file whose name is not one a class may have is
refused before anything is written: `.`, `;`, `[` and `/` each mean something else to a JVM.

A type declared in the module is a class in a package named after the module.
A variant of an algebraic data type is a class of that package too, named after its type and
then after itself, the way a nested class of Java is.
A type and one of its variants may share a name, so the type's name is part of every variant's
name and not only of the one that would otherwise clash with its own type.

The prelude types are written on every build, in the package `lumen`.
They are `Option` with `Some` and `None`, and `Result` with `Ok` and `Err`.
Writing them with the module keeps a build self-contained: there is no runtime jar to install and
no version of one to agree with.

A program of several modules is several such sets, one per module the file imports, written
beside the same source file.
Each module is its own class in its own package, so `greeting.lm` yields `greeting.class` and
`greeting/…` whatever `demo.lm` yields.
A module beside the file that nothing imports is not read and not written;
`docs/specs/modules.md` states which modules a build reaches.

## What a value is

A Lumen type is carried by a JVM type:

```text
Int             long
Bool            boolean
String          java.lang.String
a declared type the class written for it
Option<T>       lumen.Option
Result<T, E>    lumen.Result
List<T>         java.util.List
```

A type parameter written in a function signature is carried by nothing, because no method written
has one: a generic function is written once per set of types it is used at, and each of those is
carried by the type it settled on.
"How a generic is written" below says how.

A type parameter written in a type declaration is carried by `java.lang.Object`.
A generic type is one class however it is used, which is what `Option`, `Result`, and `List`
already are, so a field left open holds a reference and a whole number put in one is boxed.
Making a class per use is the same change for types that this page makes for functions, and it is
not this one.

`Int` is `long` because `Int` is 64 bits wide, which `docs/design.md` section 3 states.

`()` is carried by nothing at all.
A function whose result is `()` returns `void`, and a binding of a unit value occupies no local.
A `()` handed to something that holds a reference is the one place that leaves nothing where a
word is wanted: a field, a variant's value, or a list element a type parameter left open holds a
reference, and a `()` leaves none behind.
There a fresh `java.lang.Object` is built and stands for it.
What stands for `()` holds nothing, because `()` holds nothing, and every `()` is the same value:
a Lumen value has no identity, and `()` has no `Eq`, so nothing tells two of them apart.
Reading one back out reads a value carried by nothing, so what stood there is dropped unread.
A field typed `()` is carried by nothing wherever it appears: building a record leaves none,
and building one again from another neither reads that field nor hands it over.

A whole number and a truth value are boxed where one is put in a field left open by a type
parameter, and read back out where one is taken from such a field.
`Option`, `Result`, `List`, and a generic type the module declares all hold their values that way.
Boxing is decided from the types inference gave the expression, never from the shape of the value.

Nothing else boxes.
A call of a generic function reaches the method written for the types the call settled on, so an
`Int` crossing one is a `long`, and `docs/design.md` section 3 keeps it a type like any other.

## How a function is called

Each function of the module is a `public static` method of the module class, named as it is
written, with the descriptor its signature gives.
A module declares each name once, so no two functions share a method name.

A function is reached by `invokestatic` on the module class.
There are no function values, so nothing else calls one.

An instance's method is a method of the module class too, named for its trait, the type it is
for, and itself, joined by `$`: `Eq$Point$is_equal`.
It is written whether anything calls it or not, as a function that declares no type parameter is,
and a call of its trait's method at that type is an `invokestatic` of it.
An instance the compiler supplies has no method, and what it amounts to is written out where the
call stands; `docs/specs/traits.md` states both cases.

A function of a module the file imports is reached the same way, on that module's class.
`greeting.hello("world")` is `invokestatic greeting.hello`, with the descriptor read off the type
inference gave the use.
That is the very descriptor the other module wrote the method with, because a module offers only
functions written once and in types both modules have; `docs/specs/modules.md` states both rules.

A module that declares `main` is written with one method more: `main([Ljava/lang/String;)V`, the
shape a JVM starts at, whose whole body is a call of the `main` the module declares.
It is the one method of a module class no function wrote, and the one name a module class carries
twice, which the JVM tells apart by descriptor.
Writing the entry point with the module is what makes running the module class the same thing as
running the program, so `lumen run` supplies nothing of its own; `docs/specs/run.md` says how.

## How a list is built

A written list is gathered into a `java.lang.Object[]` and handed to `java.util.List.of`.
The array is as long as the list has elements, and it is filled left to right, so each element
is evaluated once and in the order it is written.
An empty list is the same three steps with nothing between the array and the call.

The array holds references, so a whole number or a truth value written in a list is boxed on the
way in, exactly as one put in a field a type parameter left open is.

`List.of` is what builds the list because what it gives back holds its elements and cannot be
changed, which is what a Lumen value is.
A list that was a view of the array it was gathered into would be a JVM object that something
else could still reach through, and `docs/design.md` section 2 keeps that out.
`List.of` is declared on an interface, so the call names it as one; that is the only place a
module reaches a static method of an interface.

## How a generic is written

A function that declares no type parameter is written once, named as the source names it.

A function that declares one is written once per set of types it is used at, and the one the
source wrote is written not at all: it has no descriptor, because a type parameter is carried by
nothing.
A module that never uses a generic function writes no method for it, as it writes no method for
a type parameter.
The module declaring one is therefore the only module that knows which of its methods exist,
which is why a generic is not offered through an import.

Which types a use settles is read off the type inference gave that use, and the method it reaches
is the one written for them.

Two uses are the same use when they settled every type parameter on the same type, named without
its arguments.
`identity` at `Option<Int>` and `identity` at `Option<Bool>` both settled `T` on `Option`, so one
method serves both; `identity` at `Int` and at `Bool` settled it on two types, so each has its own.

Leaving a type's arguments out is what makes the set of methods finite.
A type argument never reaches a descriptor, so nothing is lost by it, and a generic that calls
itself at a type one deeper than the one it was written for asks for a method already written.

A written method is named for the function and the types its parameters settled on, joined by
`$`: `identity$Int`, `identity$Option`, `pair$Int$String`.
A type is named by its own name, whatever it is written with, because a type argument never
reaches a descriptor.
`()` is written `$Unit` and a use that settles nothing is written `$Any`.
`$` is legal in a JVM method name and Lumen has no operator for it, so a name written this way is
one no source can collide with.

Two methods may still be written under one name, because a declared type may be called `Unit` or
`Any` and `()` is not that type.
They are told apart by their descriptors, which is how a module class already carries `main`
twice, and a call names the descriptor it reaches.
A method is the one already written only when it is called the same and takes and gives back the
same.

A body is written with each type parameter standing for what the use settled it at, so a value of
a type parameter is carried by whatever that type is carried by, and a call inside that body is a
use of its own.
A generic that a generic calls is therefore written for the types the outer one was written for.

Nothing reaches a generic from outside the module that declares it, because nothing in the
toolchain reads a second file: `docs/specs/modules.md` says a module imports only what the
compiler supplies.
Every use a generic has is therefore in the module that declares it, and every method it needs is
written by the same build.
What a use in another module would need is that module's build to have the declaring module's
body, which is a question for the change that lets one file reach another and not for this page.

## How a type is laid out

Every class written is a value class: its identity bit is clear, so the JVM may flatten a value
of it and never asks which object it is.
A Lumen value has no identity to begin with, which `docs/design.md` section 2 states, so nothing
is lost and the JVM is free to lay the value out flat wherever it can.
Every field of a value class is `final` and strict: the constructor writes each field before it
hands itself up to its base, and the value is whole by the time anything above it runs.
Strict is `ACC_STRICT_INIT`, which JEP 539 defines and JEP 401 asks of every value-class field.
The verifier holds a constructor to that order.
A branch before the call up would need a stack map frame listing the fields still unset.
No constructor written branches, so no such frame is written.
No method a module writes is `synchronized`, because locking is done on an object and a value
has no identity to be locked on; a JVM refuses a `monitorenter` on a value outright.

A record held in a record is laid out flat, which `docs/design.md` section 1 promises.
Where the value sits is the JVM's decision, and the compiler's part is to say what it must know
first, which the `LoadableDescriptors` attribute below does.
That attribute earns its place by measurement, and the change that adds it records the numbers.

A record type is a `final` value class with one field per field it declares, in the order it
declares them, and one constructor taking them in that order.
`user.name` is a `getfield`, and `user { active: false }` is a new instance built from the fields
of the old one.

A record a function builds and never lets go of is not built at all.
`point := Point { across: 1, down: 2 }` puts each field in a local of its own, and `point.across`
reads that local: no `new` is emitted, no constructor is called, and no `getfield` is read.
A Lumen value has no identity, which `docs/design.md` section 2 states, so a value split across
locals is the same value as one laid out on the heap and nothing a program can ask tells them apart.
This is what Valhalla calls scalarization, and the language meets its precondition today.

A binding is split when every one of these holds:

- it is written with `:=`, so what the name holds never changes;
- its value is a record literal naming its type, rather than an update of another record;
- every other mention of the name in the function reads one field of it.

Anything else a mention does is the value escaping: passing the name to a call, giving it back,
binding it to a second name, updating it with `point { active: false }`, or matching on it.
One escaping mention is enough to build the record, because a value that leaves the function has to
be a whole one by the time it does.
The question is asked of the function's whole body rather than of a block of it, so a name read
after the loop that built it escapes wherever it is read.

The fields are worked out in the order the type declares them, which is the order the constructor
would have taken them in.
A field carried by nothing occupies no local, the way a binding of a unit value does.
Splitting changes no result: the program computes what it computed, and only the instructions
differ.

An algebraic data type is an `abstract` value class carrying one `int` field, `tag`.
Each variant is a `final` value class extending it, whose constructor writes what the variant
carries and then passes the variant's position in the declaration up as the tag.
A variant that carries values positionally names them `value0`, `value1`, and so on.

`match` tries each arm in the order it is written.
An arm tests what it must — a tag, a whole number, a string — and falls to the next arm as soon
as one of those tests fails.
An arm that binds what a variant carries casts to the variant's class and reads its fields, and
a pattern written inside another one is tested the same way against what was read.
Exhaustiveness has already proved that some arm answers for every value, so falling past the last
arm cannot happen, and the method throws there rather than running on into the next thing.

A class written for a type declares its constructor and nothing else.
No class a module writes, that one or the module class, declares a method a JVM class inherits:
not `equals`, not `hashCode`, not `getClass`, not `toString`, and none of the rest of them.
Declaring one would put the object model back inside a Lumen value, and `docs/design.md`
section 2 declines the object model outright.
`==` is `Eq`, and a record or a variant has one only where the module writes or derives it, which
`docs/specs/traits.md` and `docs/specs/derive.md` state; the method that answers is a static
method of the module class, reached by name.
A value's hash and the text it is shown as are `Hash` and `Show`, which a type opts into the same
way, so nothing has either without asking and neither is ever the JVM's own.
A literal pattern tests a whole number, a truth value, or a string, and each of those is compared
by what it holds rather than by being one object.

Nothing a module writes asks whether two references are one object.
There is no reference comparison to emit: the one equality a lowering calls is the one `String`
declares, and `Object.equals`, which answers by identity, is never reached.

## What the bytes look like

The class-file version is 72, which is JDK 28's, and the minor version is 65535.
The minor marks a preview class file, which is what a class file holding a value class is on
JDK 28, and a JVM loads one only when started with `--enable-preview`.
`lumen run` passes that flag, so a program is run without its author knowing any of this.
Every method carries a `Code` attribute, and every `Code` attribute that branches carries a
`StackMapTable`, which the verifier requires.
A `Code` attribute carries an exception table, which is empty for every method but the one read
`docs/specs/io.md` states, and the handler's frame says the throwable alone holds where it lands.

A class carries a `LoadableDescriptors` attribute naming what its own fields hold.
A JVM settles where each field of a class sits while it loads that class, and it can fold a value
into the class holding it only where it already knows what that value holds.
JEP 401 is how a class file asks for those classes to be loaded before that, and this is that ask.
A class names the descriptor of each field whose type the same build writes, in the order the
fields are declared, and names a type two fields share once.
A field carried by a class the JVM ships, `java.lang.String` among them, is not named: it is no
value class, so nothing of it could be folded into the class holding it.
No class names itself: a type that holds a value of itself is refused before this, which
`docs/specs/types.md` states and the layout here is the reason for.
The base of a sum type is not named either: a field typed as one holds whichever variant it was
handed, so it stays a reference however early the base is loaded.
A class with no such field carries no attribute at all rather than an empty one.
Nothing else is named: what a method takes and gives back waits for a measurement that asks for it.

The output is byte-reproducible.
Two compilations of one source produce identical files, so nothing in the writer depends on a
timestamp, a build path, or the order a hash map iterates.
The constant pool is built in the order entries are first asked for, which the lowering fixes.

## The errors

Code generation raises no diagnostic.
Every way a program can be wrong has been refused by an earlier phase, and a module that reaches
this phase is written out.

## Properties

These hold and are checked with property-based tests:

1. Lowering and writing a checked module never panics and is deterministic.
2. Compiling one source twice gives byte-identical class files.
3. Every class written begins with the class-file magic and JDK 28's version, marked preview.
4. Every function a module declares that is not generic is a static method of the module class.
5. A generic function used at `Int` is written taking and giving back `long`, and boxes nothing.
6. Every constant pool entry a method refers to is within the pool.
7. No two classes of one module share a name.
8. Every method of every class ends by leaving it.
9. No class a module writes declares a method a JVM class inherits.
10. The one `equals` a module calls is the one `String` declares.
11. A record bound with `:=` and mentioned only to read its fields is lowered without a `new`.
12. Such a program computes what the same program computes when the record is built.
13. Every descriptor a class asks to load first names another class the same build writes.
14. A written list of `n` elements gathers them into an array of `n` and builds one list.
15. As many values stand for nothing as there are `()`s written where a reference is wanted.
