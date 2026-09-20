//! The invariants of `docs/specs/modules.md` and `docs/specs/packages.md`, on generated sets.

use std::collections::HashSet;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_modules::{Loaded, NotLoaded, load};

use crate::common::Beside;

/// The names a generated set of modules is drawn from, which are names an import may write.
const NAMES: [&str; 5] = ["alpha", "beta", "gamma", "delta", "epsilon"];

/// One module, written to import each of `imports` and to declare a function of its own.
fn source(imports: &[&str]) -> String {
    let mut written = String::new();
    for module in imports {
        written.push_str("import ");
        written.push_str(module);
        written.push_str("\n\n");
    }
    written.push_str("fn held() -> Int {\n    1\n}\n");
    written
}

/// A set of modules where each imports only modules written after it, which is never a ring.
fn layered(tc: &TestCase) -> Vec<(String, String)> {
    let count = tc.draw(gs::integers::<usize>().min_value(1).max_value(NAMES.len()));
    let mut modules = Vec::new();
    for (position, name) in NAMES.iter().take(count).enumerate() {
        let below: Vec<&str> = NAMES[position + 1..count]
            .iter()
            .copied()
            .filter(|_| tc.draw(gs::booleans()))
            .collect();
        modules.push(((*name).to_owned(), source(&below)));
    }
    modules
}

/// The directory those modules are written into, which is what an import looks in.
fn written(modules: &[(String, String)]) -> Beside {
    let borrowed: Vec<(&str, &str)> = modules
        .iter()
        .map(|(name, source)| (name.as_str(), source.as_str()))
        .collect();
    Beside::holding(&borrowed)
}

/// The names of every module loaded, in the order they were handed over.
fn order_of(loaded: &Loaded) -> Vec<String> {
    loaded
        .modules()
        .iter()
        .map(|module| module.name().to_owned())
        .collect()
}

#[hegel::test]
fn a_set_of_modules_that_imports_only_downwards_loads(tc: TestCase) {
    let modules = layered(&tc);
    let beside = written(&modules);
    load(&beside.file_of(&modules[0].0)).unwrap_or_else(|error| match error {
        NotLoaded::Unreadable { path, why } => panic!("{} is readable: {why}", path.display()),
        NotLoaded::Refused(refused) => panic!("it loads: {}", refused.diagnostic().message()),
    });
}

#[hegel::test]
fn every_module_is_handed_over_below_each_module_it_imports(tc: TestCase) {
    let modules = layered(&tc);
    let beside = written(&modules);
    let loaded = load(&beside.file_of(&modules[0].0)).expect("a layered set of modules loads");
    let order = order_of(&loaded);
    for module in loaded.modules() {
        let at = position_of(&order, module.name());
        for imported in imported_by(module.source()) {
            assert!(
                position_of(&order, &imported) < at,
                "{imported} is loaded before {}",
                module.name()
            );
        }
    }
}

#[hegel::test]
fn no_module_is_loaded_twice_however_many_modules_import_it(tc: TestCase) {
    let modules = layered(&tc);
    let beside = written(&modules);
    let loaded = load(&beside.file_of(&modules[0].0)).expect("a layered set of modules loads");
    let order = order_of(&loaded);
    let once: HashSet<&String> = order.iter().collect();
    assert_eq!(once.len(), order.len(), "{order:?} holds each module once");
}

#[hegel::test]
fn loading_is_deterministic(tc: TestCase) {
    let modules = layered(&tc);
    let beside = written(&modules);
    let named = beside.file_of(&modules[0].0);
    let first = load(&named).expect("a layered set of modules loads");
    let again = load(&named).expect("a layered set of modules loads");
    assert_eq!(order_of(&first), order_of(&again));
}

#[hegel::test]
fn a_refusal_points_inside_the_file_it_is_shown_against(tc: TestCase) {
    let mut modules = layered(&tc);
    let missing = source(&["nowhere"]);
    modules[0].1 = missing;
    let beside = written(&modules);
    let NotLoaded::Refused(refused) =
        load(&beside.file_of(&modules[0].0)).expect_err("an import of nothing is refused")
    else {
        panic!("the file is readable");
    };
    let span = refused.diagnostic().span();
    assert!(!span.text(refused.source()).is_empty());
}

/// Where `name` was handed over, which every module loaded has a place in.
fn position_of(order: &[String], name: &str) -> usize {
    order
        .iter()
        .position(|loaded| loaded == name)
        .unwrap_or_else(|| panic!("{name} is one of the modules loaded"))
}

/// Every module `source` imports, read back out of the text it was written from.
fn imported_by(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("import "))
        .map(str::to_owned)
        .collect()
}

/// `docs/specs/packages.md` property 1: a package's modules load below what they import.
#[hegel::test]
fn a_set_of_modules_split_across_two_packages_loads_dependencies_first(tc: TestCase) {
    let modules = layered(&tc);
    let (first, rest) = modules
        .split_first()
        .expect("a layered set holds one module");
    let held = written(rest);
    held.packaged("held", &[]);
    let app = written(std::slice::from_ref(first));
    app.packaged("app", &[&held]);
    let loaded = load(&app.file_of(&first.0)).expect("a layered set across packages loads");
    let order = order_of(&loaded);
    for module in loaded.modules() {
        let at = position_of(&order, module.name());
        for imported in imported_by(module.source()) {
            assert!(
                position_of(&order, &imported) < at,
                "{imported} is loaded before {}",
                module.name()
            );
        }
    }
}

/// `docs/specs/packages.md` property 2: a module beside the file wins over every dependency's.
#[hegel::test]
fn a_module_beside_the_importing_file_is_reached_however_many_packages_hold_one(tc: TestCase) {
    let reached = NAMES[0];
    let holders: Vec<Beside> = (0..tc.draw(gs::integers::<usize>().min_value(1).max_value(3)))
        .map(|at| {
            let holding = Beside::holding(&[(reached, &source(&[]))]);
            holding.packaged(&format!("holder{at}"), &[]);
            holding
        })
        .collect();
    let app = Beside::holding(&[("main", &source(&[reached])), (reached, &source(&[]))]);
    app.packaged("app", &holders.iter().collect::<Vec<&Beside>>());
    let loaded = load(&app.file_of("main")).expect("a module beside the file is reachable");
    assert_eq!(loaded.modules()[0].path(), app.file_of(reached));
}

/// `docs/specs/packages.md` property 3: a manifest is read or refused, and never panicked over.
#[hegel::test]
fn a_manifest_is_read_or_refused_against_the_text_it_is_written_in(tc: TestCase) {
    let lines = tc.draw(gs::vecs(gs::sampled_from(MANIFEST_LINES.to_vec())).max_size(5));
    let written: Vec<String> = lines.iter().map(|line| format!("{line}\n")).collect();
    let manifest = written.concat();
    let app = Beside::holding(&[("main", &source(&[]))]);
    app.stating(&manifest);
    let Err(NotLoaded::Refused(refused)) = load(&app.file_of("main")) else {
        return;
    };
    if refused.path() == app.directory().join(lumen_modules::MANIFEST) {
        assert_eq!(refused.source(), manifest);
        assert!(refused.diagnostic().span().end() <= manifest.len());
    }
}

/// The lines a generated manifest is built out of, well formed and otherwise.
const MANIFEST_LINES: [&str; 8] = [
    "package app",
    "version 0.2.0",
    "depends ../nowhere",
    "",
    "author nobody",
    "package",
    "package a b",
    "packageapp",
];
