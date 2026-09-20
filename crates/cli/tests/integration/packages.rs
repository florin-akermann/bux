//! `lumen check` and `lumen build` handed a package, and what an import of one reaches.
//!
//! `docs/specs/packages.md` states them: a directory is a package, every module of one is
//! checked or built, and an import that names nothing beside the file names a dependency's.

use lumen_modules::MANIFEST;

use crate::common::{Example, Sibling, Within, lumen};

/// A module that declares one function and states the example it is held to.
const DECLARING: &str = "// example: hello() == \"hi\"\nfn hello() -> String {\n    \"hi\"\n}\n";

/// A module that imports `circle` and calls the function that module declares.
const IMPORTING: &str =
    "import circle\n\n// example: said() == \"hi\"\nfn said() -> String {\n    circle.hello()\n}\n";

/// The manifest of a package called `name` that depends on each of `depends`.
fn manifest(name: &str, depends: &[&str]) -> String {
    let stated: Vec<String> = depends
        .iter()
        .map(|named| format!("depends {named}\n"))
        .collect();
    format!("package {name}\nversion 0.2.0\n{}", stated.concat())
}

/// An example whose directory is a package, holding `example.lm` and a `helper.lm` beside it.
fn package_of(module: &str, helper: &str) -> Example {
    let example = Example::new(module);
    example.beside_it(&Sibling {
        named: "helper",
        content: helper,
    });
    example.within_it(&Within {
        at: MANIFEST,
        content: &manifest("app", &[]),
    });
    example
}

/// The same run of the command, over the package rather than over one module of it.
fn over_the_package(command: &str, example: &Example) -> crate::common::Run {
    lumen(&[command, example.directory.to_str().expect("a UTF-8 path")])
}

#[test]
fn check_takes_a_package_and_accepts_every_module_it_holds() {
    let example = package_of(DECLARING, DECLARING);

    let run = over_the_package("check", &example);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn check_refuses_a_module_of_the_package_that_the_compiler_will_not_have() {
    let example = package_of(DECLARING, "fn hello( -> String {\n}\n");

    let run = over_the_package("check", &example);

    assert_eq!(run.code, 1);
    assert!(run.stderr.starts_with("error[L0100]: "), "{}", run.stderr);
}

#[test]
fn check_says_a_directory_holding_no_manifest_is_no_package() {
    let example = Example::new(DECLARING);

    let run = over_the_package("check", &example);

    assert_eq!(run.code, 2);
    assert!(
        run.stderr.contains("there is no package here"),
        "{}",
        run.stderr
    );
}

#[test]
fn check_reaches_a_module_of_a_package_the_manifest_depends_on() {
    let example = Example::new(IMPORTING);
    example.within_it(&Within {
        at: MANIFEST,
        content: &manifest("app", &["shapes"]),
    });
    example.within_it(&Within {
        at: "shapes/bux.package",
        content: &manifest("shapes", &[]),
    });
    example.within_it(&Within {
        at: "shapes/circle.lm",
        content: DECLARING,
    });

    let run = lumen(&["check", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn check_refuses_a_manifest_that_is_not_written_the_way_a_manifest_is() {
    let example = Example::new(DECLARING);
    example.within_it(&Within {
        at: MANIFEST,
        content: "package app\nauthor nobody\n",
    });

    let run = lumen(&["check", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 1);
    assert!(run.stderr.starts_with("error[L0315]: "), "{}", run.stderr);
    assert!(run.stderr.contains(MANIFEST), "{}", run.stderr);
}

#[test]
fn check_refuses_a_depends_naming_a_directory_that_holds_no_package() {
    let example = Example::new(DECLARING);
    example.within_it(&Within {
        at: MANIFEST,
        content: &manifest("app", &["nowhere"]),
    });

    let run = lumen(&["check", example.path.to_str().expect("a UTF-8 path")]);

    assert_eq!(run.code, 1);
    assert!(run.stderr.starts_with("error[L0316]: "), "{}", run.stderr);
}

#[test]
fn build_takes_a_package_and_writes_a_class_for_every_module_it_holds() {
    let example = package_of(DECLARING, DECLARING);

    let run = over_the_package("build", &example);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(example.beside("example.class").is_some());
    assert!(example.beside("helper.class").is_some());
}

#[test]
fn build_writes_nothing_of_a_package_whose_first_module_is_refused() {
    let example = package_of("fn hello( -> String {\n}\n", DECLARING);

    let run = over_the_package("build", &example);

    assert_eq!(run.code, 1);
    assert!(example.beside("example.class").is_none());
    assert!(example.beside("helper.class").is_none());
}
