//! The catalogue: every code the compiler refuses with, and the long form of each.

/// Declares the catalogue: the codes, the list of them, and what each one holds.
///
/// Writing those three out separately is what lets them drift, and the drift that matters is a
/// code the compiler can raise but `lumen explain` cannot find. Here a code cannot be added
/// without joining the list, and its long form is the file named after its number, so neither a
/// missing entry nor a mismatched file is writeable.
macro_rules! catalogue {
    ($($(#[$about:meta])* $code:ident => $number:literal,)+) => {
        /// A stable identifier for one kind of refusal.
        ///
        /// A code is assigned once and never reused, so a spec or a test cites the code rather
        /// than the prose, which is then free to be reworded. `docs/specs/diagnostics.md` says
        /// where each one is raised.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Code {
            $($(#[$about])* $code,)+
        }

        /// Every code there is, in the order their numbers run.
        ///
        /// This is what `lumen explain` searches and what the catalogue's tests walk.
        pub const CODES: &[Code] = &[$(Code::$code,)+];

        impl Code {
            /// The code as it is written, such as `L0105`.
            #[must_use]
            pub const fn number(self) -> &'static str {
                match self {
                    $(Self::$code => $number,)+
                }
            }

            /// The long form of the code, which `lumen explain` prints.
            #[must_use]
            pub const fn explanation(self) -> &'static str {
                match self {
                    $(Self::$code => include_str!(concat!("explanations/", $number, ".md")),)+
                }
            }
        }
    };
}

catalogue! {
    /// The grammar expected one thing and the source wrote another.
    UnexpectedToken => "L0100",
    /// A string has no closing quote.
    UnterminatedString => "L0101",
    /// A character is not part of the language.
    UnknownCharacter => "L0102",
    /// A number does not fit in a whole number.
    IntegerTooLarge => "L0103",
    /// A backslash is followed by something that is not an escape.
    UnknownEscape => "L0104",
    /// Comparisons are chained.
    ChainedComparison => "L0105",
    /// Brackets nest deeper than the parser descends.
    NestingTooDeep => "L0106",
    /// The file is not in canonical form.
    NotCanonical => "L0200",
    /// Nothing in scope has this name.
    UnresolvedName => "L0300",
    /// A module declares the same name twice.
    NameDeclaredTwice => "L0301",
    /// A declaration or a binding hides a name that is already in scope.
    NameShadowed => "L0302",
}

impl Code {
    /// The code that is written this way, when one is.
    #[must_use]
    pub fn written_as(written: &str) -> Option<Self> {
        CODES.iter().copied().find(|code| code.number() == written)
    }
}
