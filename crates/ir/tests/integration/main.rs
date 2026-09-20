//! The one integration-test binary of the `lumen-ir` crate.
//!
//! Every test file is a module declared here; shared helpers go in a `common` module.

mod calls;
mod common;
mod derive;
mod descriptors;
mod escaping;
mod imported;
mod io;
mod lists;
mod literals;
mod lowering;
mod operators;
mod patterns;
mod properties;
mod specialised;
mod traits;
mod unit;
