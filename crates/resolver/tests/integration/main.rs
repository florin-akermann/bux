//! The one integration-test binary of the `lumen-resolver` crate.
//!
//! Every test file is a module declared here; shared helpers go in a `common` module.

mod calls;
mod common;
mod derive;
mod errors;
mod order;
mod patterns;
mod properties;
mod scopes;
mod traits;
