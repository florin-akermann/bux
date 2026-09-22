//! The types of version 0.1, and the way each one is written in a message.

use std::fmt;

use lumen_ast::Name;
use lumen_resolver::Origin;

/// A type of version 0.1, as `docs/specs/types.md` lists them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    /// A type inference has not settled yet.
    Var(TypeVar),
    /// A type parameter a declaration wrote; it matches itself and nothing else.
    Parameter(TypeParameter),
    /// A named type with its arguments applied: `Int`, `List<User>`, `Result<User, String>`.
    Named { name: String, arguments: Vec<Type> },
    /// `(Int, String) -> Bool`.
    Function {
        parameters: Vec<Type>,
        result: Box<Type>,
    },
    /// An imported module, which a name is reached inside of rather than held as a value.
    Module(String),
    /// `()`, the type of a function that returns nothing interesting.
    Unit,
}

/// The type a value of, or nothing, is written as, which a `?` hands a `None` back from.
pub(crate) const OPTION: &str = "Option";

/// The type an attempt is written as, which a `?` hands an `Err` back from.
pub(crate) const RESULT: &str = "Result";

/// The type a run of values is written as, which the JVM holds as a `lumen.List`.
pub(crate) const LIST: &str = "List";

impl Type {
    /// A whole number, 64 bits wide, and the only one the language has.
    #[must_use]
    pub fn int() -> Self {
        Self::plain("Int")
    }

    /// A string of text.
    #[must_use]
    pub fn string() -> Self {
        Self::plain("String")
    }

    /// `true` or `false`.
    #[must_use]
    pub fn boolean() -> Self {
        Self::plain("Bool")
    }

    /// A list of `item`, which is what a `for … in` walks.
    #[must_use]
    pub fn list(item: Self) -> Self {
        Self::applied("List", vec![item])
    }

    /// A value of type `value`, or nothing.
    #[must_use]
    pub fn option(value: Self) -> Self {
        Self::applied(OPTION, vec![value])
    }

    /// A `value` when it worked and an `error` when it did not.
    #[must_use]
    pub fn result(value: Self, error: Self) -> Self {
        Self::applied(RESULT, vec![value, error])
    }

    /// Whether this names a type another module declares, reached through that module's name.
    ///
    /// The name of one holds a dot, which no name a module declares ever does: a name is one
    /// word, and `docs/specs/modules.md` reaches another module's type through the dot.
    pub(crate) fn is_of_another_module(&self) -> bool {
        matches!(self, Self::Named { name, .. } if name.contains('.'))
    }

    /// A function from `parameters` to `result`.
    #[must_use]
    pub fn function(parameters: Vec<Self>, result: Self) -> Self {
        Self::Function {
            parameters,
            result: Box::new(result),
        }
    }

    fn plain(name: &str) -> Self {
        Self::applied(name, Vec::new())
    }

    fn applied(name: &str, arguments: Vec<Self>) -> Self {
        Self::Named {
            name: name.to_owned(),
            arguments,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(_) => f.write_str("_"),
            Self::Parameter(parameter) => f.write_str(&parameter.name),
            Self::Named { name, arguments } if arguments.is_empty() => f.write_str(name),
            Self::Named { name, arguments } => write!(f, "{name}<{}>", listed(arguments)),
            Self::Function { parameters, result } => {
                write!(f, "({}) -> {result}", listed(parameters))
            }
            Self::Module(name) => f.write_str(name),
            Self::Unit => f.write_str("()"),
        }
    }
}

/// A stand-in for a type inference has not settled, numbered in the order they were made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeVar(pub(crate) u32);

/// A type parameter, known by where it came from so that two of one name stay apart.
///
/// `T` in `fn identity<T>(value: T) -> T` is one of these inside that function's body, which is
/// what makes a body that returns an `Int` a mismatch rather than a proof.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeParameter {
    pub name: String,
    pub origin: Origin,
}

impl TypeParameter {
    /// The parameter `written` between the angle brackets of a declaration.
    #[must_use]
    pub fn written(written: &Name) -> Self {
        Self {
            name: written.text.clone(),
            origin: Origin::Declared(written.span),
        }
    }

    /// A parameter of a type the prelude supplies, such as the `T` of `Option<T>`.
    #[must_use]
    pub fn prelude(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            origin: Origin::Prelude,
        }
    }

    /// A stand-in nothing names, which stands where a type is carried by what every value fits.
    ///
    /// A function whose type inference left free is generic without writing a type parameter,
    /// and `docs/specs/codegen.md` erases such a stand-in rather than writing a method per type.
    /// This is that stand-in, so the type a module reaching one writes is the type it reaches.
    pub(crate) fn erased() -> Self {
        Self {
            name: "_".to_owned(),
            origin: Origin::Prelude,
        }
    }
}

/// The types as a message lists them, separated by commas.
fn listed(types: &[Type]) -> String {
    types
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ")
}
