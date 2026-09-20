//! The one integration-test binary of the `lumen-types` crate.
//!
//! Every test file is a module declared here; shared helpers go in a `common` module.

mod arguments;
mod common;
mod discarding;
mod errors;
mod flags;
mod generics;
mod holds;
mod imports;
mod inference;
mod predicate;
mod properties;
mod supplied;
