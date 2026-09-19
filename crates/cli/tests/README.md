# Tests of the `lumen` crate

`integration/` is one Cargo test binary; each test file is a module declared in `main.rs`.
Shared helpers live in `integration/common.rs` and are reached with `use crate::common;`.
