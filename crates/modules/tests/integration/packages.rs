//! What a manifest states, and what an import of a module of a dependency reaches.
//!
//! `docs/specs/packages.md` states them: an import is answered beside the importing file first
//! and out of a package that file's package depends on second, and a module two of them hold is
//! refused rather than picked between.

use std::path::{Path, PathBuf};

use lumen_diagnostics::Code;
use lumen_modules::{Loaded, Module, NotLoaded, load};

use crate::common::Beside;

/// A module that imports each of `names`, in the order it names them, and declares one function.
fn importing(names: &[&str]) -> String {
    let written: Vec<String> = names
        .iter()
        .map(|name| format!("import {name}\n\n"))
        .collect();
    format!("{}fn main() -> () {{\n}}\n", written.concat())
}

/// A module that declares one function and imports nothing.
const DECLARING: &str = "fn hello() -> String {\n    \"hi\"\n}\n";

/// The names of every module loaded, in the order they are handed over.
fn order_of(loaded: &Loaded) -> Vec<&str> {
    loaded.modules().iter().map(Module::name).collect()
}

/// The refusal loading `named` out of `beside` gives, which every one of these expects.
fn refused(beside: &Beside, named: &str) -> (Code, String) {
    match load(&beside.file_of(named)) {
        Ok(_) => panic!("{named} is refused"),
        Err(NotLoaded::Unreadable { path, why }) => {
            panic!("{} is readable: {why}", path.display())
        }
        Err(NotLoaded::Refused(refused)) => (
            refused.diagnostic().code(),
            refused.diagnostic().message().to_owned(),
        ),
    }
}

/// The file `package` holds `module` in, spelled the way `app`'s manifest reaches it.
fn reached(app: &Beside, package: &Beside, module: &str) -> PathBuf {
    app.directory()
        .join("..")
        .join(package.named())
        .join(module)
        .with_extension("lm")
}

/// The refusal two files claiming one name is refused with, in the words the reader sees.
fn both(module: &str, first: &Path, second: &Path) -> String {
    format!(
        "`{module}` is both `{}` and `{}`",
        first.display(),
        second.display()
    )
}

#[test]
fn an_import_reaches_a_module_of_a_package_the_manifest_depends_on() {
    let shapes = Beside::holding(&[("circle", DECLARING)]);
    shapes.packaged("shapes", &[]);
    let app = Beside::holding(&[("main", &importing(&["circle"]))]);
    app.packaged("app", &[&shapes]);
    let loaded = load(&app.file_of("main")).expect("a module of a dependency is reachable");
    assert_eq!(order_of(&loaded), ["circle", "main"]);
}

#[test]
fn a_module_of_a_dependency_is_read_out_of_that_package_s_own_directory() {
    let shapes = Beside::holding(&[("circle", DECLARING)]);
    shapes.packaged("shapes", &[]);
    let app = Beside::holding(&[("main", &importing(&["circle"]))]);
    app.packaged("app", &[&shapes]);
    let loaded = load(&app.file_of("main")).expect("a module of a dependency is reachable");
    assert_eq!(loaded.modules()[0].source(), DECLARING);
}

#[test]
fn a_module_beside_the_importing_file_is_reached_before_one_in_a_dependency() {
    let shapes = Beside::holding(&[("circle", DECLARING)]);
    shapes.packaged("shapes", &[]);
    let app = Beside::holding(&[
        ("main", &importing(&["circle"])),
        ("circle", "fn hello() -> String {\n    \"beside\"\n}\n"),
    ]);
    app.packaged("app", &[&shapes]);
    let loaded = load(&app.file_of("main")).expect("a module beside the file is reachable");
    assert_eq!(loaded.modules()[0].path(), app.file_of("circle"));
}

#[test]
fn a_module_of_a_dependency_reaches_that_package_s_own_dependencies() {
    let deep = Beside::holding(&[("point", DECLARING)]);
    deep.packaged("geometry", &[]);
    let shapes = Beside::holding(&[("circle", &importing(&["point"]))]);
    shapes.packaged("shapes", &[&deep]);
    let app = Beside::holding(&[("main", &importing(&["circle"]))]);
    app.packaged("app", &[&shapes]);
    let loaded = load(&app.file_of("main")).expect("a dependency reaches its own dependencies");
    assert_eq!(order_of(&loaded), ["point", "circle", "main"]);
}

#[test]
fn a_dependency_of_a_dependency_is_not_reached_through_the_one_between_them() {
    let deep = Beside::holding(&[("point", DECLARING)]);
    deep.packaged("geometry", &[]);
    let shapes = Beside::holding(&[("circle", DECLARING)]);
    shapes.packaged("shapes", &[&deep]);
    let app = Beside::holding(&[("main", &importing(&["point"]))]);
    app.packaged("app", &[&shapes]);
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::NoSuchModule);
    assert_eq!(message, "there is no module named `point`");
}

#[test]
fn a_module_two_dependencies_both_hold_is_refused_rather_than_picked_between() {
    let shapes = Beside::holding(&[("circle", DECLARING)]);
    shapes.packaged("shapes", &[]);
    let colours = Beside::holding(&[("circle", DECLARING)]);
    colours.packaged("colours", &[]);
    let app = Beside::holding(&[("main", &importing(&["circle"]))]);
    app.packaged("app", &[&shapes, &colours]);
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::ModuleIsTwoFiles);
    assert_eq!(
        message,
        both(
            "circle",
            &reached(&app, &shapes, "circle"),
            &reached(&app, &colours, "circle")
        )
    );
}

#[test]
fn a_ring_of_imports_across_two_packages_is_refused() {
    let shapes = Beside::holding(&[("circle", &importing(&["main"]))]);
    let app = Beside::holding(&[("main", &importing(&["circle"]))]);
    shapes.packaged("shapes", &[&app]);
    app.packaged("app", &[&shapes]);
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::RingOfImports);
    assert_eq!(message, "`main` imports `circle`, which imports `main`");
}

#[test]
fn a_depends_naming_a_directory_that_holds_no_manifest_is_refused() {
    let app = Beside::holding(&[("main", &importing(&["circle"]))]);
    app.stating("package app\nversion 0.2.0\ndepends ../nowhere\n");
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::NoSuchPackage);
    assert_eq!(message, "there is no package in `../nowhere`");
}

#[test]
fn a_manifest_that_states_no_package_is_refused() {
    let app = Beside::holding(&[("main", DECLARING)]);
    app.stating("version 0.2.0\n");
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::NotAManifest);
    assert_eq!(message, "`package` is what a manifest states here");
}

#[test]
fn a_manifest_that_states_no_version_is_refused() {
    let app = Beside::holding(&[("main", DECLARING)]);
    app.stating("package app\n");
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::NotAManifest);
    assert_eq!(message, "`version` is what a manifest states here");
}

#[test]
fn a_manifest_line_that_is_no_depends_is_refused() {
    let app = Beside::holding(&[("main", DECLARING)]);
    app.stating("package app\nversion 0.2.0\nauthor nobody\n");
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::NotAManifest);
    assert_eq!(message, "`depends` is what a manifest states here");
}

#[test]
fn a_manifest_keyword_with_no_word_after_it_is_refused() {
    let app = Beside::holding(&[("main", DECLARING)]);
    app.stating("package \nversion 0.2.0\n");
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::NotAManifest);
    assert_eq!(message, "`package` states one word, and this line does not");
}

#[test]
fn a_manifest_word_holding_a_space_is_refused() {
    let app = Beside::holding(&[("main", DECLARING)]);
    app.stating("package the app\nversion 0.2.0\n");
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::NotAManifest);
    assert_eq!(message, "`package` states one word, and this line does not");
}

#[test]
fn a_refusal_of_a_manifest_points_at_the_line_it_is_about() {
    let manifest = "package app\nversion 0.2.0\nauthor nobody\n";
    let app = Beside::holding(&[("main", DECLARING)]);
    app.stating(manifest);
    let Err(NotLoaded::Refused(refused)) = load(&app.file_of("main")) else {
        panic!("a manifest that is not one is refused");
    };
    assert_eq!(refused.source(), manifest);
    assert_eq!(refused.diagnostic().span().text(manifest), "author nobody");
    assert_eq!(
        refused.path(),
        app.directory().join(lumen_modules::MANIFEST)
    );
}

#[test]
fn a_directory_with_no_manifest_reaches_what_sits_beside_it_and_nothing_else() {
    let shapes = Beside::holding(&[("circle", DECLARING)]);
    shapes.packaged("shapes", &[]);
    let app = Beside::holding(&[("main", &importing(&["circle"]))]);
    let (code, _) = refused(&app, "main");
    assert_eq!(code, Code::NoSuchModule);
}

#[test]
fn a_package_that_depends_on_nothing_reaches_what_a_bare_directory_reaches() {
    let app = Beside::holding(&[("main", &importing(&["circle"])), ("circle", DECLARING)]);
    app.packaged("app", &[]);
    let loaded = load(&app.file_of("main")).expect("a package reaches its own modules");
    assert_eq!(order_of(&loaded), ["circle", "main"]);
}

#[test]
fn a_dependency_holding_a_name_this_package_also_holds_is_refused_whichever_loads_first() {
    let shapes = Beside::holding(&[("helper", &importing(&["circle"])), ("circle", DECLARING)]);
    shapes.packaged("shapes", &[]);
    let app = Beside::holding(&[
        ("main", &importing(&["helper", "circle"])),
        ("circle", DECLARING),
    ]);
    app.packaged("app", &[&shapes]);
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::ModuleIsTwoFiles);
    assert_eq!(
        message,
        both(
            "circle",
            &reached(&app, &shapes, "circle"),
            &app.file_of("circle")
        )
    );
}

#[test]
fn a_package_two_packages_both_depend_on_is_one_module_however_it_is_reached() {
    let deep = Beside::holding(&[("point", DECLARING)]);
    deep.packaged("geometry", &[]);
    let shapes = Beside::holding(&[("circle", &importing(&["point"]))]);
    shapes.packaged("shapes", &[&deep]);
    let app = Beside::holding(&[("main", &importing(&["circle", "point"]))]);
    app.packaged("app", &[&shapes, &deep]);
    let loaded = load(&app.file_of("main")).expect("one package reached twice is one module");
    assert_eq!(order_of(&loaded), ["point", "circle", "main"]);
}

#[test]
fn a_manifest_depending_on_one_directory_twice_is_refused() {
    let shapes = Beside::holding(&[("circle", DECLARING)]);
    shapes.packaged("shapes", &[]);
    let app = Beside::holding(&[("main", &importing(&["circle"]))]);
    app.packaged("app", &[&shapes, &shapes]);
    let (code, message) = refused(&app, "main");
    assert_eq!(code, Code::NotAManifest);
    assert_eq!(
        message,
        format!("`../{}` is depended on twice", shapes.named())
    );
}

#[test]
fn a_manifest_written_with_carriage_returns_points_at_the_line_it_is_about() {
    let app = Beside::holding(&[("main", &importing(&["circle"]))]);
    app.stating("package app\r\nversion 0.2.0\r\nauthor nobody\r\n");
    let Err(NotLoaded::Refused(refusal)) = load(&app.file_of("main")) else {
        panic!("a manifest holding a line that is not one is refused");
    };
    assert_eq!(refusal.diagnostic().code(), Code::NotAManifest);
    assert_eq!(
        refusal.diagnostic().span().text(refusal.source()),
        "author nobody"
    );
}
