//! What loading does with a name the library holds, which is to read the source it carries.
//!
//! `docs/specs/library.md` states the rules these hold: a library module is read out of the
//! compiler rather than out of a file, a file of that name beside the importer is not consulted,
//! and the prelude is not among the names an import reaches.

use lumen_modules::{Loaded, Module, NotLoaded, load};
use lumen_resolver::library;

use crate::common::Beside;

/// A module importing `strings`, which is the library module version 0.1 offers.
const IMPORTING: &str = "import strings\n\nfn main() -> () {\n}\n";

/// A module importing `process`, which is the one library module that imports another.
const IMPORTING_PROCESS: &str = "import process\n\nfn main() -> () {\n}\n";

/// A module importing the prelude, which is a name no import reaches.
const IMPORTING_THE_PRELUDE: &str = "import prelude\n\nfn main() -> () {\n}\n";

#[test]
fn an_import_of_a_library_module_reaches_the_source_the_compiler_carries() {
    let beside = Beside::holding(&[("main", IMPORTING)]);
    let loaded = load(&beside.file_of("main")).expect("a library module needs no file");

    assert_eq!(source_of(&loaded, "strings"), library::source_of("strings"));
}

#[test]
fn a_file_of_that_name_beside_the_importer_does_not_shadow_the_library() {
    let beside = Beside::holding(&[
        ("main", IMPORTING),
        ("strings", "fn join() -> String {\n    \"beside\"\n}\n"),
    ]);
    let loaded = load(&beside.file_of("main")).expect("a library module needs no file");

    assert_eq!(source_of(&loaded, "strings"), library::source_of("strings"));
}

#[test]
fn a_library_module_is_handed_over_below_the_module_that_imports_it() {
    let beside = Beside::holding(&[("main", IMPORTING)]);
    let loaded = load(&beside.file_of("main")).expect("a library module needs no file");

    assert_eq!(order_of(&loaded), ["strings", "main"]);
}

#[test]
fn every_library_module_a_file_imports_is_handed_over_below_it() {
    let beside = Beside::holding(&[(
        "main",
        "import files\n\nimport io\n\nfn main() -> () {\n}\n",
    )]);
    let loaded = load(&beside.file_of("main")).expect("a library module needs no file");

    assert_eq!(order_of(&loaded), ["files", "io", "main"]);
    assert_eq!(source_of(&loaded, "io"), library::source_of("io"));
    assert_eq!(source_of(&loaded, "files"), library::source_of("files"));
}

#[test]
fn the_prelude_is_not_a_name_an_import_reaches() {
    let beside = Beside::holding(&[("main", IMPORTING_THE_PRELUDE)]);
    let refused = load(&beside.file_of("main")).expect_err("the prelude is imported by nothing");

    assert_eq!(refusal(&refused), "there is no module named `prelude`");
}

#[test]
fn a_file_called_prelude_beside_the_importer_is_not_what_that_import_reaches() {
    let beside = Beside::holding(&[
        ("main", IMPORTING_THE_PRELUDE),
        ("prelude", "fn mine() -> Int {\n    1\n}\n"),
    ]);
    let refused = load(&beside.file_of("main")).expect_err("the prelude is imported by nothing");

    assert_eq!(refusal(&refused), "there is no module named `prelude`");
}

#[test]
fn a_root_module_named_as_the_library_is_no_ring_for_the_module_that_imports_it() {
    let beside = Beside::holding(&[
        ("strings", "import helper\n\nfn main() -> () {\n}\n"),
        ("helper", IMPORTING),
    ]);
    let loaded = load(&beside.file_of("strings")).expect("the import reaches the library");

    assert_eq!(source_of(&loaded, "helper"), Some(IMPORTING));
}

#[test]
fn every_import_a_module_the_library_carries_writes_names_another_it_carries() {
    for (name, source) in library::carried() {
        for imported in source
            .lines()
            .filter_map(|line| line.strip_prefix("import "))
        {
            assert!(
                library::imported(imported).is_some(),
                "library/{name}.lm imports `{imported}`, which no import reaches"
            );
        }
    }
}

#[test]
fn a_library_module_that_imports_another_is_handed_over_below_the_one_it_imports() {
    let beside = Beside::holding(&[("main", IMPORTING_PROCESS)]);
    let loaded = load(&beside.file_of("main")).expect("a library module needs no file");

    assert_eq!(order_of(&loaded), ["list", "process", "main"]);
}

/// The names of every module loaded, in the order they are handed over.
fn order_of(loaded: &Loaded) -> Vec<&str> {
    loaded.modules().iter().map(Module::name).collect()
}

/// The text the module called `name` was read from, when loading read one of that name.
fn source_of<'a>(loaded: &'a Loaded, name: &str) -> Option<&'a str> {
    loaded
        .modules()
        .iter()
        .find(|module| module.name() == name)
        .map(Module::source)
}

/// What a refusal of loading says, which is one line without the code or the place.
fn refusal(refused: &NotLoaded) -> String {
    let NotLoaded::Refused(refusal) = refused else {
        panic!("an import of a name nothing holds is refused rather than unreadable")
    };
    refusal.diagnostic().message().to_owned()
}
