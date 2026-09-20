//! The one integration-test binary of the `lumen-ir` crate.
//!
//! Every test file is a module declared here; shared helpers go in a `common` module.

mod common;
mod descriptors;
mod escaping;
mod io;
mod lists;
mod lowering;
mod properties;
mod unit;
