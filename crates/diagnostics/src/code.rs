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
                    $(Self::$code => include_str!(concat!("../../../compiler/explanations/", $number, ".md")),)+
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
    /// Something other than a name is written on the left of `=` or `+=`.
    AssignedToValue => "L0107",
    /// A call names some of its arguments and not others.
    PartlyNamedCall => "L0108",
    /// The file is not in canonical form.
    NotCanonical => "L0200",
    /// An import is written after a declaration, or two imports are out of sort.
    ImportOutOfOrder => "L0201",
    /// A declared name is spelled some way other than the one canonical form spells it.
    NotCanonicalCase => "L0202",
    /// A declared name is an initial rather than a word a reader can look for.
    NameIsAnInitial => "L0203",
    /// Nothing in scope has this name.
    UnresolvedName => "L0300",
    /// A module declares the same name twice.
    NameDeclaredTwice => "L0301",
    /// A declaration or a binding hides a name that is already in scope.
    NameShadowed => "L0302",
    /// A declaration is written above something that uses it.
    DefinitionBeforeUse => "L0303",
    /// A name that is not a value, such as a function or a module, is written as one.
    NotAValue => "L0304",
    /// An assignment names something other than a `var` binding.
    NotAVariable => "L0305",
    /// An import names a module no file beside the importing one holds.
    NoSuchModule => "L0306",
    /// A ring of imports, which leaves the modules in it no order to be compiled in.
    RingOfImports => "L0307",
    /// A trait already has an instance for the type a second instance names.
    InstanceDeclaredTwice => "L0308",
    /// An instance does not write exactly the methods its trait declares.
    InstanceMethods => "L0309",
    /// Something that is not a trait is written where a trait belongs.
    NotATrait => "L0310",
    /// A trait is written where a type belongs.
    TraitAsType => "L0311",
    /// A derive names a trait no type derives, or a type this module does not declare.
    NotDerivable => "L0312",
    /// A type or a pattern is reached through a name that is no module.
    NotAModule => "L0313",
    /// A name that binds is written inside an or-pattern, which binds nothing.
    BindsInsideOr => "L0314",
    /// A manifest states something other than `package`, `version`, and a `depends` for each one.
    NotAManifest => "L0315",
    /// A directory named as a package holds no manifest, so there is no package there.
    NoSuchPackage => "L0316",
    /// Two packages a module can reach both hold a module of the name an import writes.
    ModuleIsTwoFiles => "L0317",
    /// An instance writes an argument of its type that is not a type parameter it declares.
    NotATypeParameter => "L0318",
    /// A type met a type it does not match.
    TypeMismatch => "L0400",
    /// A call passes more or fewer arguments than the function takes.
    WrongArgumentCount => "L0401",
    /// A field is reached that the type reached through does not have.
    UnknownField => "L0402",
    /// A type would have to contain itself.
    InfiniteType => "L0403",
    /// A record is built without one of the fields it declares.
    MissingField => "L0404",
    /// A record is written with one of its fields given a value twice.
    FieldWrittenTwice => "L0405",
    /// An operator is written over a type that has no instance of the trait that operator is.
    NoOperator => "L0406",
    /// A divisor is written as zero, which the compiler can see has no answer.
    DivisorIsZero => "L0407",
    /// A statement leaves a value behind and nothing takes it.
    Discarded => "L0408",
    /// A call passes its arguments positionally where the declaration repeats a type.
    Unnamed => "L0409",
    /// An argument is named something other than the parameter it is passed for.
    Misnamed => "L0410",
    /// A call names the arguments of something that has no parameter names to write.
    Unnameable => "L0411",
    /// A parameter is a bare `Bool`, so a call of it passes `true` and says no more.
    FlagParameter => "L0412",
    /// A function whose result is `Bool` is named for a command rather than a question.
    NotAPredicate => "L0413",
    /// A module does not declare the name reached inside it.
    NotInModule => "L0414",
    /// A declared type holds a value of itself, which no whole value could ever be.
    HoldsItself => "L0415",
    /// A trait method is used at a type that has no instance of that trait.
    NoInstance => "L0418",
    /// A parameter of a trait's method states no type, which nothing is left to infer.
    SignatureWithoutType => "L0419",
    /// A whole number does not fit the type it is written at, which its instance says it holds.
    LiteralDoesNotFit => "L0420",
    /// A bound of an `IntegerLiteral` instance is written as other than one whole number.
    BoundIsNotAWholeNumber => "L0421",
    /// A type derives `Eq` and holds a value of a type that has none.
    HeldTypeHasNoInstance => "L0422",
    /// A call written with its first argument in front names its arguments.
    NamedInFront => "L0423",
    /// A generic of another module is constrained by a trait that stays in that module.
    TraitStaysInItsModule => "L0424",
    /// An `extern` signature names a type no Java member takes or gives back.
    DoesNotCross => "L0425",
    /// An `extern` states something that is no Java name.
    NotAJavaName => "L0426",
    /// A derive names a type an `extern type` declares, whose contents are the JVM's.
    DerivesAForeignType => "L0427",
    /// An `extern method` or an `extern new` reaches a class, and its signature names none.
    ReachesNoClass => "L0428",
    /// An `extern` writes a width where there is no `Int` to widen to or to narrow from.
    WidensNoInt => "L0429",
    /// An `extern new` gives back a type an `extern type` named an interface.
    BuildsAnInterface => "L0430",
    /// An `extern` narrows a parameter and gives back something other than an `Option`.
    NarrowsWithoutOption => "L0431",
    /// A `match` leaves a value of the type it matches unanswered.
    NonExhaustiveMatch => "L0500",
    /// A `match` lists its arms in an order the type does not declare its variants in.
    ArmOutOfOrder => "L0501",
    /// A hole is still in the program, and a hole has nothing to compile.
    HoleBuilt => "L0600",
    /// A function a module declares at the top level states no example.
    NoExample => "L0601",
    /// An example is written where nothing carries one.
    ExampleDocumentsNothing => "L0602",
    /// An example a module states did not hold when it was run.
    ExampleDoesNotHold => "L0603",
    /// A run of the examples reaches a module whose name the module under test declares.
    ExampleRunReachesTheName => "L0604",
}

impl Code {
    /// The code that is written this way, when one is.
    #[must_use]
    pub fn written_as(written: &str) -> Option<Self> {
        CODES.iter().copied().find(|code| code.number() == written)
    }
}
