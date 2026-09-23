//! The one integration-test binary of the `lumen-parser` crate.
//!
//! Every test file is a module declared here; shared helpers live in `common`, and the printed
//! form of a tree lives in `printed`, which the harness of the parser written in Bux reads too.

mod common;
mod errors;
mod examples;
mod expressions;
mod items;
mod patterns;
mod printed;
mod properties;
mod statements;
