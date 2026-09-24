
Packages
--------

A package is a directory that holds a manifest called `bux.package`, and every package has one
layout. Its modules are the `.bx` files at the top of `src/`, its test modules are those at the
top of `tests/`, and a build writes its classes into `target/`, beside the manifest.

    shapes/
        bux.package
        src/circle.bx
        tests/rounded.bx
        target/

A `.bx` file beside the manifest is refused as `L0323`, and so is a manifest inside `src/` or
`tests/`. A test module reaches a module of `src/` by its name, and a module of `src/` never
reaches a test module. A file in no package is a bare module, and it writes `target/` beside
itself.

Handed a directory rather than a file, `bux check` and `bux build` run over every module the
package holds: those of `src/` first, then those of `tests/`, each part in the order the names
sort. They stop at the first refusal. `bux test` runs the examples and the tests of every module,
reports them in the same order, and does not stop at the first module that fails; with no path, it
runs the package in the current directory. A directory holding no manifest is no package, and a
command handed one says so and stops with exit code 2.

    package shapes
    version 0.2.0
    depends ../geometry
    jar lib/geometry.jar sha256:4f2c000000000000000000000000000000000000000000000000000000009a1e

Every line is a keyword, one space, and one word, except a `jar` line. `package` is written
first and `version` second, a `depends` line for each dependency comes after both, and a `jar`
line for each Java archive comes last. A path is read against the directory the manifest sits
in, so `../geometry` is that directory's sibling.

An import that names no module beside the file that wrote it names a module in `src/` of a
package the manifest depends on, and never one of its `tests/`. Nothing is fetched: the directory
is already there, or the compiler says there is no package in it. A module the library carries is
reached before either, and a module beside the importing file before a dependency's.

Two files claiming a module of one name are refused rather than picked between, whether they are
two dependencies' or one this package holds beside a dependency's that something already reached:
one name has one definition, and an order would make which definition a name means depend on the
order the files were reached in. A directory depended on twice is refused for the same reason
there is one way to write a manifest.

A `jar` line names a Java archive by its path and by the SHA-256 of its bytes: `sha256:` and 64
lowercase hexadecimal digits, as `shasum -a 256` shows them. The path holds no blank, `:`, or `*`.
Nothing is fetched: the archive is a file on disk already. `bux build`, `bux run`, and `bux test`
refuse an archive that is not there, that has another hash, that is no Java archive, or whose
manifest has a `Class-Path`, because a JVM reaches each archive a `Class-Path` names. A run puts
the stated archives on the class path after `target/`, and a package with no `jar` line reaches
no archive. A program reaches a class of an archive with an `extern`.

The name and the version are stated and nothing reads either yet. A manifest that leaves one out
is refused all the same, because a package says what it is before anything asks.

`bux fmt`, `bux run`, and `bux api` each take a file. A package has no canonical text of its
own, no `main` to run, and no surface beyond its modules'.
