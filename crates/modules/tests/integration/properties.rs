//! The invariants of `docs/specs/modules.md`, checked on generated sets of modules.

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
