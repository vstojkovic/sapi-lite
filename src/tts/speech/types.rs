use std::fmt::Display;
use std::hash::Hash;

/// Provides a hint about how to pronounce the associated content.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum SayAs<'s> {
    /// Pronounce a sequence of numbers as a date, e.g. "03/08/2000" as "march eighth two thousand".
    DateMDY,
    /// Pronounce a sequence of numbers as a date, e.g. "03/08/2000" as "august third two thousand".
    DateDMY,
    /// Pronounce a sequence of numbers as a date, e.g. "2000/08/03" as "march eighth two thousand".
    DateYMD,
    /// Pronounce a sequence of numbers as a year and month, e.g. "2000/03" as "march two thousand".
    DateYM,
    /// Pronounce a sequence of numbers as a month and year, e.g. "03/2000" as "march two thousand".
    DateMY,
    /// Pronounce a sequence of numbers as a day and month, e.g. "03/08" as "march eighth".
    DateDM,
    /// Pronounce a sequence of numbers as a month and day, e.g. "03/08" as "august third".
    DateMD,
    /// Pronounce a number as a year, e.g. "1979" as "nineteen seventy-nine".
    DateYear,
    /// Pronounce a sequence of numbers as a time, e.g. "10:24" as "ten twenty-four".
    Time,
    /// Pronounce a number as a cardinal number, e.g. "1024" as "one thousand twenty-four".
    NumberCardinal,
    /// Pronounce a number as a sequence of digits, e.g. "1024" as "one zero two four".
    NumberDigit,
    /// Pronounce a number as a fraction, e.g. "3/8" as "three eighths".
    NumberFraction,
    /// Pronounce a number as a fraction, e.g. "10.24" as "ten point two four".
    NumberDecimal,
    /// Pronounce a sequence of numbers as a telephone, e.g. "(206) 555-1234" as "two zero six five
    /// five five one two three four".
    PhoneNumber,
    /// A custom pronunciation hint supported by the engine.
    Custom(&'s str),
}

impl<'s> SayAs<'s> {
    pub(super) fn sapi_id(&self) -> &str {
        match self {
            Self::DateMDY => "date_mdy",
            Self::DateDMY => "date_dmy",
            Self::DateYMD => "date_ymd",
            Self::DateYM => "date_ym",
            Self::DateMY => "date_my",
            Self::DateDM => "date_dm",
            Self::DateMD => "date_md",
            Self::DateYear => "date_year",
            Self::Time => "time",
            Self::NumberCardinal => "number_cardinal",
            Self::NumberDigit => "number_digit",
            Self::NumberFraction => "number_fraction",
            Self::NumberDecimal => "number_decimal",
            Self::PhoneNumber => "phone_number",
            Self::Custom(s) => s,
        }
    }
}

macro_rules! decl_clamped_int {
    {$(#[$meta:meta])* $name:ident($base:ty) in $min:literal..$max:literal} => {
        $(#[$meta])*
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        pub struct $name($base);

        impl $name {
            /// Clamps the given value to the interval
            #[doc = concat!("[", stringify!($min), ", ", stringify!($max), "]")]
            /// and constructs a new instance from it.
            pub fn new(value: $base) -> Self {
                Self(value.clamp($min, $max))
            }

            /// Returns the value encapsulated by this instance.
            pub fn value(&self) -> $base {
                self.0
            }
        }

        impl From<$base> for $name {
            fn from(source: $base) -> Self {
                Self::new(source)
            }
        }

        impl From<$name> for $base {
            fn from(source: $name) -> Self {
                source.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

decl_clamped_int! {
    /// Voice pitch, represented as a value in the interval [-10, 10], with 0 being normal pitch.
    Pitch(i32) in -10..10
}

decl_clamped_int! {
    /// Speech rate, represented as a value in the interval [-10, 10], with 0 being normal speed.
    Rate(i32) in -10..10
}

decl_clamped_int! {
    /// Voice volume, represented as a value in the interval [0, 100], with 100 being full volume.
    Volume(u32) in 0..100
}

impl Volume {
    pub(crate) fn from_sapi(source: u16) -> Self {
        Self::new(source as _)
    }

    pub(crate) fn sapi_value(&self) -> u16 {
        self.0 as _
    }
}
