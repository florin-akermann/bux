//! Helpers shared by the loader's behaviour tests.
//!
//! Each test writes its modules into a directory of its own under the system's temporary
//! directory, so no two tests can see each other's files and nothing is written into the
//! repository.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use lumen_modules::MANIFEST;

/// A directory of Lumen modules, which is what an import looks in.
pub struct Beside {
    directory: PathBuf,
}

impl Beside {
    /// A directory no other test writes to, holding each `(name, source)` as its own module.
    pub fn holding(modules: &[(&str, &str)]) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "lumen-modules-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&directory).expect("a temporary directory is creatable");
        let beside = Self { directory };
        for (name, source) in modules {
            std::fs::write(beside.file_of(name), source).expect("a module is writable");
        }
        beside
    }

    /// The file the module called `name` is written in.
    pub fn file_of(&self, name: &str) -> PathBuf {
        self.directory.join(name).with_extension("lm")
    }

    /// The directory itself, which is where a module that is not written would have been.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// The manifest of a package called `name` that depends on each of `depends`.
    pub fn packaged(&self, name: &str, depends: &[&Self]) -> &Self {
        let stated: Vec<String> = depends
            .iter()
            .map(|package| format!("depends ../{}\n", package.named()))
            .collect();
        self.stating(&format!(
            "package {name}\nversion 0.2.0\n{}",
            stated.concat()
        ))
    }

    /// The manifest that makes this directory a package, written exactly as `manifest` states it.
    pub fn stating(&self, manifest: &str) -> &Self {
        std::fs::write(self.directory.join(MANIFEST), manifest).expect("a manifest is writable");
        self
    }

    /// What this directory is called, which is what a manifest beside it writes to reach it.
    ///
    /// Every one of these sits directly under the system's temporary directory, so a sibling of
    /// one is reached from it by name.
    pub fn named(&self) -> String {
        self.directory
            .file_name()
            .expect("a temporary directory has a name")
            .to_string_lossy()
            .into_owned()
    }
}

impl Drop for Beside {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

static NEXT: AtomicUsize = AtomicUsize::new(0);
