//! What a derive names, and what it is refused for, which `docs/specs/derive.md` states.

use crate::common::{refusal, resolved};

/// A record with a derive above it, which is the shape every example here varies.
const USER: &str = "type User = {\n    id: Int\n}\n";

/// `derived` above [`USER`], as the module a derive is read in.
fn module(derived: &str) -> String {
    format!("{derived}\n\n{USER}")
}

/// The four traits a type derives, which `docs/specs/derive.md` states are the standard ones.
const DERIVABLE: [&str; 4] = ["Eq", "Ord", "Hash", "Show"];

#[test]
fn a_derive_of_each_standard_trait_for_a_type_this_module_declares_resolves() {
    for of in DERIVABLE {
        let source = module(&format!("derive {of} for User"));

        assert_eq!(resolved(&source).program().items.len(), 2, "{of}");
    }
}

#[test]
fn a_derive_naming_every_standard_trait_at_once_resolves() {
    let source = module(&format!("derive {} for User", DERIVABLE.join(", ")));

    assert_eq!(resolved(&source).program().items.len(), 2);
}

#[test]
fn a_derive_of_a_trait_no_type_derives_is_refused_naming_that_trait() {
    let error = refusal(&module("derive Add for User"));

    assert_eq!(error.message(), "`Add` is not a trait a type derives");
    assert_eq!(
        error.help(),
        "`Eq`, `Ord`, `Hash`, and `Show` are the traits a type derives; write others by hand"
    );
}

#[test]
fn a_derive_names_the_first_trait_it_lists_that_no_type_derives() {
    let error = refusal(&module("derive Eq, Add for User"));

    assert_eq!(error.message(), "`Add` is not a trait a type derives");
}

#[test]
fn a_derive_for_a_type_this_module_does_not_declare_is_refused() {
    let error = refusal(&module("derive Eq for Int"));

    assert_eq!(error.message(), "`Int` is not a type this module declares");
    assert_eq!(
        error.help(),
        "a derive reads the declaration it names, so it names one this module writes"
    );
}

#[test]
fn a_derive_of_something_that_is_not_a_trait_is_refused_as_an_instance_is() {
    let error = refusal(&module("derive User for User"));

    assert_eq!(error.message(), "`User` is not a trait");
}

#[test]
fn a_derive_of_a_trait_the_type_already_has_an_instance_of_is_refused() {
    let written = concat!(
        "instance Eq<User> {\n    fn is_equal(one, other) -> Bool {\n",
        "        true\n    }\n}"
    );
    let error = refusal(&format!("derive Eq for User\n\n{written}\n\n{USER}"));

    assert_eq!(error.message(), "`Eq` already has an instance for `User`");
}

#[test]
fn a_derive_written_twice_for_one_type_is_the_same_instance_twice() {
    let error = refusal(&module("derive Eq for User\n\nderive Eq for User"));

    assert_eq!(error.message(), "`Eq` already has an instance for `User`");
}

#[test]
fn a_derive_below_the_type_it_names_is_refused_as_every_use_below_one_is() {
    let error = refusal(&format!("{USER}\nderive Eq for User\n"));

    assert_eq!(
        error.message(),
        "`User` is written above `derive Eq for User`, which uses it"
    );
}
