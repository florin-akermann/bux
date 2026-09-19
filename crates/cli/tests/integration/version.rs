//! The binary reports the crate version it was built from.

use std::process::Command;

#[test]
fn version_flag_prints_the_crate_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .arg("--version")
        .output()
        .expect("the lumen binary runs");

    assert!(output.status.success(), "lumen --version exits 0");
    let stdout = String::from_utf8(output.stdout).expect("version output is UTF-8");
    assert_eq!(
        stdout.trim(),
        format!("lumen {}", env!("CARGO_PKG_VERSION"))
    );
}
