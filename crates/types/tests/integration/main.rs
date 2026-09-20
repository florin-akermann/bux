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
mod literals;
mod operators;
mod predicate;
mod propagating;
mod properties;
mod supplied;
mod traits;
