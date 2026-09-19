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

A record type is a `final` value class with one field per field it declares, in the order it
declares them, and one constructor taking them in that order.
`user.name` is a `getfield`, and `user { active: false }` is a new instance built from the fields
of the old one.

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

Two values are compared by what they hold rather than by being one object.
`User { id: 1 }` equals another `User` built the same way, which identity would deny, so every
class a module writes declares the `equals` that `==` and a literal pattern call.

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
