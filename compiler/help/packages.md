
Packages
--------

A package is a directory of modules with a manifest called `bux.package` beside them. Handed a
directory rather than a file, `bux check` and `bux build` run over every module the package
holds, in the order their names sort, and stop at the first refusal. A directory holding no
manifest is no package, and a command handed one says so and stops with exit code 2.

    package shapes
    version 0.2.0
    depends ../geometry

Every line is a keyword, one space, and one word. `package` is written first and `version`
second, and a `depends` line for each dependency comes after both. A path is read against the
directory the manifest sits in, so `../geometry` is that directory's sibling.

An import that names no module beside the file that wrote it names a module of a package the
manifest depends on. Nothing is fetched: the directory is already there, or the compiler says
there is no package in it. A module the library carries is reached before either, and a module
beside the importing file before a dependency's.

Two files claiming a module of one name are refused rather than picked between, whether they are
two dependencies' or one this package holds beside a dependency's that something already reached:
one name has one definition, and an order would make which definition a name means depend on the
order the files were reached in. A directory depended on twice is refused for the same reason
there is one way to write a manifest.

The name and the version are stated and nothing reads either yet. A manifest that leaves one out
is refused all the same, because a package says what it is before anything asks.

`bux fmt`, `bux run`, `bux test`, and `bux api` each take a file. A package has no
canonical text of its own, no `main` to run, no examples, and no surface beyond its modules'.
