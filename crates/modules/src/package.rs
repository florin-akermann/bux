//! The manifest that names a package, and the packages a module of one may reach.
//!
//! `docs/specs/packages.md` states the manifest line by line and the order an import is answered
//! in. A directory holding no manifest is no package, which is every directory until one is
//! written, so a module in none reaches exactly what sits beside it.

use std::fmt;
use std::path::{Path, PathBuf};

use lumen_ast::Span;

use crate::NotLoaded;
use crate::error::{LoadError, LoadErrorKind};

/// What a package's manifest is called, written beside the modules of the package it names.
pub const MANIFEST: &str = "bux.package";

/// What a module is written in, which is how a package's modules are found.
pub const SUFFIX: &str = "lm";

/// A path as the imports reached it, and as the file system holds it.
///
/// The two differ because two routes spell one file differently: a package that two packages
/// both depend on is reached through each of them. What a refusal shows is the route the author
/// wrote, and what says whether two routes are one file is the file system's own answer.
#[derive(Clone, Debug)]
pub(crate) struct Route {
    shown: PathBuf,
    found: PathBuf,
}

/// Every package the one in `directory` depends on, which is none where there is no package there.
///
/// # Errors
///
/// Returns a manifest that is not written the way a manifest is written, here or in one of the
/// packages it depends on, a `depends` naming a directory that holds no manifest at all, and a
/// directory two `depends` both name.
pub(crate) fn reachable_from(directory: &Path) -> Result<Vec<Route>, NotLoaded> {
    let Some(manifest) = read(directory)? else {
        return Ok(Vec::new());
    };
    let mut depends: Vec<Route> = Vec::new();
    for named in stated_by(&manifest)? {
        let one = depended_on(&manifest, directory, &named)?;
        if depends.iter().any(|held| held.is_the_same_as(&one)) {
            let twice = LoadErrorKind::DependsTwice(named.word.clone());
            return Err(manifest.refusing(&LoadError::at(named.span, twice)));
        }
        depends.push(one);
    }
    Ok(depends)
}

/// The package `named` names, read against `directory`, having proved there is one there.
///
/// Its manifest is read and thrown away: reading it is what says a package is there at all, and
/// nothing about a dependency is read until one of its modules is.
fn depended_on(manifest: &Manifest, directory: &Path, named: &At) -> Result<Route, NotLoaded> {
    let at = directory.join(&named.word);
    let Some(held) = read(&at)? else {
        let nothing = LoadErrorKind::NoSuchPackage(named.word.clone());
        return Err(manifest.refusing(&LoadError::at(named.span, nothing)));
    };
    stated_by(&held)?;
    Ok(Route::to(at))
}

impl Route {
    /// The route to `shown`, asking the file system what it is before anything compares it.
    ///
    /// A path naming nothing is its own answer, which is what a module the compiler carries has:
    /// it is written in no file, and no route ever reaches it.
    pub(crate) fn to(shown: PathBuf) -> Self {
        let found = std::fs::canonicalize(&shown).unwrap_or_else(|_| shown.clone());
        Self { shown, found }
    }

    /// The path as it was reached, which is what a refusal about it names.
    pub(crate) fn shown(&self) -> &Path {
        &self.shown
    }

    /// Whether both routes reach one file, however differently the two are spelled.
    pub(crate) fn is_the_same_as(&self, other: &Self) -> bool {
        self.found == other.found
    }
}

/// One manifest as it was read: the file it is, and the text a refusal about it points into.
struct Manifest {
    path: PathBuf,
    source: String,
}

impl Manifest {
    /// `error` as a refusal of this manifest, shown against the file the manifest is.
    fn refusing(&self, error: &LoadError) -> NotLoaded {
        NotLoaded::refused(&error.diagnostic(), &self.path, &self.source)
    }
}

/// The manifest `directory` holds, or nothing at all where it holds none.
fn read(directory: &Path) -> Result<Option<Manifest>, NotLoaded> {
    let path = directory.join(MANIFEST);
    if !path.is_file() {
        return Ok(None);
    }
    match std::fs::read_to_string(&path) {
        Ok(source) => Ok(Some(Manifest { path, source })),
        Err(why) => Err(NotLoaded::Unreadable { path, why }),
    }
}

/// One word a manifest states, and the line it states it on.
struct At {
    word: String,
    span: Span,
}

/// Where each dependency `manifest` states sits, or the first line of it that is not a line a
/// manifest writes.
///
/// The name and the version are read here and kept nowhere. Nothing compares two versions while
/// nothing is fetched, and a refusal names the file a reader opens rather than the package a file
/// belongs to. Both are required all the same: a package says what it is before anything asks.
fn stated_by(manifest: &Manifest) -> Result<Vec<At>, NotLoaded> {
    read_out_of(&manifest.source).map_err(|error| manifest.refusing(&error))
}

/// The same, against the text alone, which is where every refusal of a manifest is decided.
fn read_out_of(source: &str) -> Result<Vec<At>, LoadError> {
    let mut lines = lines_of(source).into_iter();
    next_one(&mut lines, Keyword::Package, source)?;
    next_one(&mut lines, Keyword::Version, source)?;
    lines.map(|line| word_of(&line, Keyword::Depends)).collect()
}

/// The word the next line states after `keyword`, the manifest having to have such a line.
fn next_one(
    lines: &mut impl Iterator<Item = Line>,
    keyword: Keyword,
    source: &str,
) -> Result<At, LoadError> {
    let Some(line) = lines.next() else {
        let whole = Span::new(0, source.len());
        return Err(LoadError::at(whole, LoadErrorKind::LineIsNot(keyword)));
    };
    word_of(&line, keyword)
}

/// The one word `line` states after `keyword`, or why it is not the line that was wanted.
fn word_of(line: &Line, keyword: Keyword) -> Result<At, LoadError> {
    let stated = line
        .text
        .strip_prefix(keyword.written())
        .and_then(|rest| rest.strip_prefix(' '));
    let Some(word) = stated else {
        return Err(LoadError::at(line.span, LoadErrorKind::LineIsNot(keyword)));
    };
    if word.is_empty() || word.contains(char::is_whitespace) {
        return Err(LoadError::at(line.span, LoadErrorKind::NotOneWord(keyword)));
    }
    Ok(At {
        word: word.to_owned(),
        span: line.span,
    })
}

/// One line of a manifest, and where in the file it is written.
struct Line {
    text: String,
    span: Span,
}

/// Every line of `source`, blank ones and all, each with the span it covers.
///
/// The line break is counted as the bytes it is rather than as one byte, so a manifest written
/// with a carriage return before each break points at the line it is about like any other.
fn lines_of(source: &str) -> Vec<Line> {
    let mut at = 0;
    let mut lines = Vec::new();
    for held in source.split_inclusive('\n') {
        let text = held.trim_end_matches('\n').trim_end_matches('\r');
        lines.push(Line {
            text: text.to_owned(),
            span: Span::new(at, text.len()),
        });
        at += held.len();
    }
    lines
}

/// What a line of a manifest opens with, which is what says which line it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Keyword {
    Package,
    Version,
    Depends,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.written())
    }
}

impl Keyword {
    /// The word the line opens with, which a line of a manifest is read by.
    const fn written(self) -> &'static str {
        match self {
            Self::Package => "package",
            Self::Version => "version",
            Self::Depends => "depends",
        }
    }
}
