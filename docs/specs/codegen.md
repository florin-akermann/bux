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
a type parameter java.lang.Object
```

`Int` is `long` because `Int` is 64 bits wide, which `docs/design.md` section 3 states.

`()` is carried by nothing at all.
A function whose result is `()` returns `void`, and a binding of a unit value occupies no local.
Version 0.1 can write `()` but can do nothing with one, so a representation would never be read.

A type parameter erases to `java.lang.Object`, so a call that passes an `Int` or a `Bool` where a
parameter is a type parameter boxes it, and one that reads such a result back at a settled type
unboxes it.
Boxing is decided from the types inference gave the call, never from the shape of the value.

## How a function is called

Each function of the module is a `public static` method of the module class, named as it is
written, with the descriptor its signature gives.
A module declares each name once, so no two functions share a method name.

A function is reached by `invokestatic` on the module class.
Version 0.1 has no function values, so nothing else calls one.

A module that declares `main` is written with one method more: `main([Ljava/lang/String;)V`, the
shape a JVM starts at, whose whole body is a call of the `main` the module declares.
It is the one method of a module class no function wrote, and the one name a module class carries
twice, which the JVM tells apart by descriptor.
Writing the entry point with the module is what makes running the module class the same thing as
running the program, so `lumen run` supplies nothing of its own; `docs/specs/run.md` says how.

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
`==` is `Eq`, which version 0.1 gives to `Int`, `Bool`, and `String` alone, so nothing asks a
record or a variant whether it is the same as another one.
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
4. Every method a module declares is written as a static method of the module class.
5. Every constant pool entry a method refers to is within the pool.
6. No two classes of one module share a name.
7. Every method of every class ends by leaving it.
8. No class a module writes declares a method a JVM class inherits.
9. The one `equals` a module calls is the one `String` declares.
10. A record bound with `:=` and mentioned only to read its fields is lowered without a `new`.
11. Such a program computes what the same program computes when the record is built.
