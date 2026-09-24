# Packages

A package is a directory of modules with a name and a version, and a dependency is another one.

## Intent

`docs/implementation.md` section 10 names package management, and no document says what a package
is.
A module is the file beside the file that imports it, so a program reaches nothing it does not
sit next to, and a library someone else wrote cannot be reached at all.
This spec says what a package is, what its manifest states, and how an import reaches a module of
one.

Nothing is fetched.
A dependency is a directory that is already there, named by the manifest of the package that
depends on it.
A registry, a lockfile, and a version resolved rather than stated each wait for a requirement
offline-first does not have.

## What a package is

A package is a directory holding a manifest called `bux.package`.
Its modules are the `.bx` files in that directory, each named by its file as every module is.
A package has no subdirectories of modules: one directory holds one package's modules.

A file in no package is a module still, and reaches what sits beside it exactly as before.
A manifest adds dependencies and archives and nothing else.
So a package with neither reaches what a bare directory reaches.

## The manifest

`bux.package` is read as lines, and every line is a keyword, one space, and one word.
A `jar` line is the one exception, and the next section states it.

```text
package shapes
version 0.2.0
depends ../geometry
```

`package` states the name the package is called, and is written first.
`version` states which version it is, and is written second.
`depends` states the directory of a package this one depends on, and every one is written after.
A path is read against the directory the manifest sits in, so `../geometry` is that directory's
sibling.

There is one way to write a manifest.
The order above is the only order, and a `jar` line comes after every `depends` line.
A line that is none of `package`, `version`, `depends`, and `jar` is refused.
A word holds no space, so a directory whose name holds one is not one a `depends` names.

The name and the version are stated and nothing reads either yet.
Comparing two versions needs two packages of one name to reach, and nothing reaches a second copy
of a package while nothing is fetched.
A refusal names the file a reader opens rather than the package that file belongs to, because a
file is what they go and change.
Both are required all the same: a package says what it is before anything asks, and the version
that resolves is the version that fetches.

A directory is depended on once.
Two `depends` naming one directory is `L0315`, because the second states nothing the first did
not, and there is one way to write a manifest.

## Java archives

A `jar` line names a Java archive by its path and its hash.
A program reaches a class of the archive with an `extern`, which `docs/specs/interop.md` states.

```text
package shapes
version 0.2.0
depends ../geometry
jar lib/geometry.jar sha256:4f2c000000000000000000000000000000000000000000000000000000009a1e
```

The line is `jar`, one space, the path, one space, and the hash.
The path is read against the directory of the package, as the path of a `depends` is.
A path holds no blank and no `:`, because a class path puts a `:` between two archives.
A path holds no `*`, because a JVM reads a class path entry that ends in `*` as many archives.
An archive is stated once, so two `jar` lines with one path are `L0315`.
The hash is `sha256:` and 64 lowercase hexadecimal digits.
It is the SHA-256 of the bytes of the file, which `shasum -a 256` also shows.
Every `jar` line comes after every `depends` line, so each kind of line is in one block.

Nothing is fetched.
The archive is a file on disk already, as a dependency is a directory on disk already.
A package with no `jar` line reaches no archive.

A build, a run, and a test hold each archive to its line before they write a class.
The file is there, it has the hash that the line states, and it opens as a Java archive.
Its manifest has no `Class-Path`, because a JVM reaches each archive that a `Class-Path` names.
Such an archive is refused whatever its `Class-Path` holds, so a run reaches only stated archives.
The hash pins the bytes, so a changed archive is refused until its line states the new hash.

The archives of a dependency are reached too, and `docs/specs/run.md` states the order.
A package reaches its own archives first, then those of each `depends`, in line order.
A package that two routes reach adds its archives once, at the first place.

The compiler does not read a class out of an archive, copy an archive, or write a manifest.
The line is for the JVM target alone, and `docs/implementation.md` section 1 says so.

The package reader in `compiler/modules.bx` reads the line.
`compiler/archives.bx` holds the externs that open an archive, because only a build opens one.
So the library stays small, and no program reaches a Java archive through it.
`compiler/digest.bx` is the SHA-256, written in Bux, over the bytes that `files.read_bytes` reads.
It is written in Bux because no array crosses the boundary, which `docs/specs/interop.md` states.

## How an import reaches a module

`import demo` is answered by the first of these that holds `demo`:

1. What the library carries, which `docs/specs/library.md` lists.
2. `demo.bx` beside the file that wrote the import.
3. `demo.bx` in a package the manifest beside that file depends on.

Nothing else is looked in, and an import that reaches none of them is `L0306`.

A package's own modules therefore win over its dependencies', and the library's win over both.
That is an order a reader can hold: what the compiler has, then what this package has, then what
it was given.

Which file an import reaches is settled before anything else about it is.
A module already loaded under that name is the module the import reaches only where it is that
same file, and a build reads that file once however many modules import it.
Two files claiming one name are `L0317` whichever of them was read first, so what an import
reaches never depends on which module the walk happened to reach first.
Two routes to one file are one file: a package that two packages both depend on is reached
through each of them and is read once.

The manifest that is read is the one beside the file that wrote the import, never the one beside
the file a command named.
A module of a dependency reaches that dependency's own dependencies and none of this package's,
because a module is read without knowing which module imported it.

A dependency of a dependency is not reached through the one between them.
`demo` is a module of a package this one depends on, or it is nothing, so a package states every
package its own modules import.

Two files claiming `demo` are refused as `L0317`, whether they are two dependencies' or one this
package holds beside a dependency's that something already reached.
One name has one definition, which `docs/design.md` section 16 states, and picking one of the two
would make which definition a name means depend on the order they were reached in.
A package therefore does not hold a module of a name one of its dependencies holds and uses.

A ring of imports across packages is `L0307`, exactly as a ring within one is.
Loading follows an import into a dependency the way it follows one beside, so a module is
compiled after everything it imports wherever that came from.

## What a command takes

`bux check` and `bux build` take a package as well as a file.
A directory is a package, and either command run over one runs over every module the package
holds, in the order their names sort, stopping at the first refusal.
Each of them is compiled as the module a command named, so a module several of them import is
compiled once for each, and a package is as many compilations as it has modules.
That is what a package costs until a measurement says the sharing is worth building.

`bux fmt`, `bux run`, `bux test`, and `bux api` each take a file.
A package has no canonical text of its own, no `main` to run, no examples, and no surface beyond
its modules'; each of those is a module's and is asked of the module.

A directory holding no manifest is no package, and a command handed one says so and stops with
exit code 2, as it does for a directory it cannot list.

## The errors

| name                  | code    | message                                                |
|-----------------------|---------|--------------------------------------------------------|
| line is not that line | `L0315` | `version` is what a manifest states here               |
| word is not one word  | `L0315` | `package` states one word, and this line does not      |
| depended on twice     | `L0315` | `../geometry` is depended on twice                     |
| no package there      | `L0316` | there is no package in `../geometry`                   |
| module is two files   | `L0317` | `demo` is both `../shapes/demo.bx` and `demo.bx`       |
| not an archive line   | `L0315` | `jar` states a path and a hash, and this line does not |
| archive stated twice  | `L0315` | `lib/x.jar` is stated twice                            |
| archive is not there  | `L0606` | `lib/x.jar` is not there                               |
| hash is another       | `L0607` | `lib/x.jar` has the hash `sha256:…`, and this line states another |
| names more archives   | `L0608` | `lib/x.jar` names more archives in its `Class-Path`    |
| no Java archive       | `L0609` | `lib/x.jar` is no Java archive                         |

A line that is not the line belonging there helps with the first text below.
A word that is missing or holds a space helps with `a manifest line is a keyword, one space, and
one word`.
A directory depended on twice helps with ``a manifest states one `depends` for each package it
reaches``.
`L0316` helps with ``a package is a directory holding `bux.package```.
`L0317` helps with `one name has one definition; rename one of the two modules`.
A malformed `jar` line, an archive stated twice, and `L0606` to `L0609` help with the texts below.
The texts are in that order, after the first text.

```text
a manifest is `package`, then `version`, then a `depends` for each dependency, then a `jar` for each archive
a `jar` line is `jar`, a path, and `sha256:` with 64 lowercase hexadecimal digits
a manifest states one `jar` for each archive it reaches
a `jar` line names a file on disk already, and nothing is fetched
a hash pins the bytes of an archive; state the hash of the archive the package uses
a run reaches only the archives a manifest states; use an archive with no `Class-Path`
a `jar` line names a zip file of classes, as the `jar` tool of the JDK writes one
```

`L0606` to `L0609` each point at the `jar` line, and a build stops at the first archive refused.
`L0607` shows the hash that the file has, so the reader sees both hashes.

`L0315` refuses a line the manifest has no place for, a keyword whose word is missing or holds a
space, and a directory two `depends` both name.
It also refuses a `jar` line that is not `jar`, a path, and a hash, and a repeated `jar` line.
It points at the line, and at the whole manifest where the line it wanted is not written at all.
A line break is the bytes it is written as, so a manifest written with a carriage return before
each one points at its lines like any other.

`L0316` refuses a `depends` naming a directory that holds no manifest, pointing at that line.
A command handed such a directory has no manifest to point into, so that is said about the
directory the way a file that cannot be read is said about.

`L0317` points at the import, in the file that wrote it, and names both files, each as the route
the imports reached it along spells it.
Two modules of one package never clash, because one directory holds one file of a name.

`L0306` and `L0307` are unchanged.
An import that reaches nothing and a ring of imports are refused wherever the module would have
come from.

## Properties

These hold and are checked by drawn properties in the runner:

1. A package whose modules import only downwards loads, dependencies before dependents.
2. A module beside the importing file is the one an import reaches, however many dependencies
   hold one of that name.
3. Reading a manifest never panics, and a refusal of one points inside the manifest it is about.
4. A printed `jar` line reads back as the path and the hash it was printed with, in line order.
5. Each malformed shape of a `jar` line is refused as `L0315`, at the line itself.
6. The SHA-256 of `compiler/digest.bx` is the one that `shasum -a 256` gives for a drawn file.

`tests/pinned.bx` holds properties 4 to 6.
The runner skips property 6 with the reason when `shasum` cannot be run.
`tests/archived.bx` packs a class with the `jar` tool of the JDK, and holds each refusal.
It shows that `run` and `test` reach the class, and that they reach no unstated archive.
It shows that a dependency's archive is reached too.
The runner skips it with the reason when `JAVA_HOME` names no JDK.
