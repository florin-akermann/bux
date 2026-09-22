//! The one integration-test binary of the `lumen-ir` crate.
//!
//! Every test file is a module declared here; shared helpers go in a `common` module.

mod branching;
mod calls;
mod common;
mod crossing;
mod derive;
mod descriptors;
mod escaping;
mod imported;
mod lists;
mod literals;
mod lowering;
mod operators;
mod patterns;
mod properties;
mod reaching;
mod specialised;
mod traits;
mod unit;
