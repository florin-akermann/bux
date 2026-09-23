//! The one integration-test binary of the `lumen-jvm` crate.
//!
//! Every test file is a module declared here; shared helpers go in a `common` module.

mod common;
mod files;
mod frames;
mod loading;
mod modules;
mod properties;
mod reader;
