//! What the JVM is told a value is.

use std::fmt;

/// What one value is, which is what a field descriptor says.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Descriptor {
    /// A whole number, which is what `Int` is carried by.
    Long,
    /// A truth value, which is what `Bool` is carried by.
    Boolean,
    /// A small whole number, which nothing in Lumen is carried by; a variant's tag is one.
    Integer,
    Reference(ClassName),
}

impl Descriptor {
    /// A reference to the class written this way.
    #[must_use]
    pub fn reference(class: &str) -> Self {
        Self::Reference(ClassName::new(class))
    }

    /// How many local slots and stack words this takes, which is two for a `long`.
    #[must_use]
    pub const fn width(&self) -> u16 {
        if self.is_wide() { 2 } else { 1 }
    }

    /// Whether this takes two slots rather than one, which only a whole number does.
    #[must_use]
    pub const fn is_wide(&self) -> bool {
        matches!(self, Self::Long)
    }
}

impl fmt::Display for Descriptor {
    /// The descriptor as a class file writes it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Long => f.write_str("J"),
            Self::Boolean => f.write_str("Z"),
            Self::Integer => f.write_str("I"),
            Self::Reference(class) => write!(f, "L{class};"),
        }
    }
}

/// What a method takes and gives back, where nothing given back is `void`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MethodDescriptor {
    pub parameters: Vec<Descriptor>,
    pub result: Option<Descriptor>,
}

impl MethodDescriptor {
    /// The method taking `parameters` and giving back `result`.
    #[must_use]
    pub const fn new(parameters: Vec<Descriptor>, result: Option<Descriptor>) -> Self {
        Self { parameters, result }
    }

    /// The slots the parameters occupy, which is what a body's locals start after.
    #[must_use]
    pub fn width(&self) -> u16 {
        self.parameters.iter().map(Descriptor::width).sum()
    }
}

impl fmt::Display for MethodDescriptor {
    /// The descriptor as a class file writes it, such as `(JZ)Ljava/lang/String;`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("(")?;
        for parameter in &self.parameters {
            write!(f, "{parameter}")?;
        }
        f.write_str(")")?;
        match &self.result {
            Some(result) => write!(f, "{result}"),
            None => f.write_str("V"),
        }
    }
}

/// A class, in the form a class file writes it: `java/lang/String`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClassName(String);

impl ClassName {
    /// The class written this way, whose parts are separated by `/`.
    #[must_use]
    pub fn new(written: &str) -> Self {
        Self(written.to_owned())
    }

    /// The name as a class file writes it.
    #[must_use]
    pub fn written(&self) -> &str {
        &self.0
    }

    /// The path of the file this class is written to, relative to where a build writes.
    #[must_use]
    pub fn path(&self) -> String {
        format!("{}.class", self.0)
    }
}

impl fmt::Display for ClassName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
