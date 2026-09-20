//! The one integration-test binary of the `lumen-ir` crate.
//!
//! Every test file is a module declared here; shared helpers go in a `common` module.

mod common;
mod derive;
mod descriptors;
mod escaping;
mod io;
mod lists;
mod literals;
mod lowering;
mod operators;
mod properties;
mod traits;
mod unit;
