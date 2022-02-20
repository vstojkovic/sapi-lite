use std::ffi::OsString;
use std::str::FromStr;

use strum_macros::{EnumString, IntoStaticStr};

use crate::token::{Category, Token};
use crate::Result;

/// Specifies the age of a voice.
#[derive(Debug, EnumString, IntoStaticStr)]
#[strum(ascii_case_insensitive)]
#[allow(missing_docs)]
pub enum VoiceAge {
    Adult,
    Child,
    Senior,
    Teen,
}

/// Specifies the gender of a voice.
#[derive(Debug, EnumString, IntoStaticStr)]
#[strum(ascii_case_insensitive)]
#[allow(missing_docs)]
pub enum VoiceGender {
    Female,
    Male,
    Neutral,
}

/// A voice installed on the system.
pub struct Voice {
    pub(crate) token: Token,
}

impl Voice {
    /// Returns the name of this voice.
    pub fn name(&self) -> Option<OsString> {
        self.token.attr("name").ok()
    }

    /// Returns the age of this voice.
    pub fn age(&self) -> Option<VoiceAge> {
        self.token
            .attr("age")
            .ok()
            .as_ref()
            .and_then(|s| s.to_str())
            .and_then(|s| VoiceAge::from_str(s).ok())
    }

    /// Returns the gender of this voice.
    pub fn gender(&self) -> Option<VoiceGender> {
        self.token
            .attr("gender")
            .ok()
            .as_ref()
            .and_then(|s| s.to_str())
            .and_then(|s| VoiceGender::from_str(s).ok())
    }

    /// Returns the language of this voice.
    pub fn language(&self) -> Option<OsString> {
        self.token.attr("language").ok()
    }
}

/// Encapsulates the criteria for selecting a voice.
pub struct VoiceSelector {
    sapi_expr: String,
}

impl VoiceSelector {
    /// Creates a new, empty selector.
    pub fn new() -> Self {
        Self {
            sapi_expr: String::new(),
        }
    }

    /// Returns a selector that requires the voice to have the given name, along with all the
    /// previously specified conditions.
    pub fn name_eq<S: AsRef<str>>(self, name: S) -> Self {
        self.append_condition("name=", name.as_ref())
    }

    /// Returns a selector that requires the voice to have a name different from the one given here,
    /// along with all the previously specified conditions.
    pub fn name_ne<S: AsRef<str>>(self, name: S) -> Self {
        self.append_condition("name!=", name.as_ref())
    }

    /// Returns a selector that requires the voice to have the given age, along with all the
    /// previously specified conditions.
    pub fn age_eq(self, age: VoiceAge) -> Self {
        self.append_condition("age=", age.into())
    }

    /// Returns a selector that requires the voice to have an age different from the one given here,
    /// along with all the previously specified conditions.
    pub fn age_ne(self, age: VoiceAge) -> Self {
        self.append_condition("age!=", age.into())
    }

    /// Returns a selector that requires the voice to have the given gender, along with all the
    /// previously specified conditions.
    pub fn gender_eq(self, gender: VoiceGender) -> Self {
        self.append_condition("gender=", gender.into())
    }

    /// Returns a selector that requires the voice to have a gender different from the one given
    /// here, along with all the previously specified conditions.
    pub fn gender_ne(self, gender: VoiceGender) -> Self {
        self.append_condition("gender!=", gender.into())
    }

    /// Returns a selector that requires the voice to have the given language, along with all the
    /// previously specified conditions.
    pub fn language_eq<S: AsRef<str>>(self, language: S) -> Self {
        self.append_condition("language=", language.as_ref())
    }

    /// Returns a selector that requires the voice to have a language different from the one given
    /// here, along with all the previously specified conditions.
    pub fn language_ne<S: AsRef<str>>(self, language: S) -> Self {
        self.append_condition("language!=", language.as_ref())
    }

    fn append_condition(mut self, prefix: &str, val: &str) -> Self {
        if !self.sapi_expr.is_empty() {
            self.sapi_expr.push(';')
        }
        self.sapi_expr.push_str(prefix);
        self.sapi_expr.push_str(val);
        self
    }

    pub(crate) fn into_sapi_expr(self) -> String {
        self.sapi_expr
    }
}

/// If successful, returns an iterator enumerating all the installed voices that satisfy the given
/// criteria.
///
/// All returned voices will satisfy the `required` criteria. The voices that satisfy the
/// `optional` criteria will be returned before the rest.
pub fn installed_voices(
    required: Option<VoiceSelector>,
    optional: Option<VoiceSelector>,
) -> Result<impl Iterator<Item = Voice>> {
    let category = Category::new(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Speech\Voices")?;
    let tokens = category.enum_tokens(
        required.map(VoiceSelector::into_sapi_expr),
        optional.map(VoiceSelector::into_sapi_expr),
    )?;

    Ok(tokens.map(|token| Voice {
        token,
    }))
}
