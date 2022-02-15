use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum SayAs<'s> {
    DateMDY,
    DateDMY,
    DateYMD,
    DateYM,
    DateMY,
    DateDM,
    DateMD,
    DateYear,
    Time,
    NumberCardinal,
    NumberDigit,
    NumberFraction,
    NumberDecimal,
    PhoneNumber,
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
    {$name:ident($base:ty) in $min:literal..$max:literal} => {
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        pub struct $name($base);

        impl $name {
            pub fn new(value: $base) -> Self {
                Self(value.clamp($min, $max))
            }

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

decl_clamped_int! { Pitch(i32) in -10..10 }
decl_clamped_int! { Rate(i32) in -10..10 }
decl_clamped_int! { Volume(u32) in 0..100 }

impl Volume {
    pub(crate) fn from_sapi(source: u16) -> Self {
        Self::new(source as _)
    }

    pub(crate) fn sapi_value(&self) -> u16 {
        self.0 as _
    }
}
