use std::ffi::OsString;
use std::str::FromStr;

use strum_macros::{EnumString, IntoStaticStr};

use crate::Result;
use crate::token::{Token, Category};

#[derive(Debug, EnumString, IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum VoiceAge {
    Adult,
    Child,
    Senior,
    Teen,
}

#[derive(Debug, EnumString, IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum VoiceGender {
    Female,
    Male,
    Neutral,
}

pub struct Voice {
    pub(crate) token: Token,
}

impl Voice {
    pub fn name(&self) -> Option<OsString> {
        self.token.attr("name").ok()
    }

    pub fn age(&self) -> Option<VoiceAge> {
        self.token
            .attr("age")
            .ok()
            .as_ref()
            .and_then(|s| s.to_str())
            .and_then(|s| VoiceAge::from_str(s).ok())
    }

    pub fn gender(&self) -> Option<VoiceGender> {
        self.token
            .attr("gender")
            .ok()
            .as_ref()
            .and_then(|s| s.to_str())
            .and_then(|s| VoiceGender::from_str(s).ok())
    }

    pub fn language(&self) -> Option<OsString> {
        self.token.attr("language").ok()
    }
}

pub struct VoiceSelector {
    sapi_expr: String,
}

impl VoiceSelector {
    pub fn new() -> Self {
        Self {
            sapi_expr: String::new(),
        }
    }

    pub fn name_eq<S: AsRef<str>>(self, name: S) -> Self {
        self.append_condition("name=", name.as_ref())
    }

    pub fn name_ne<S: AsRef<str>>(self, name: S) -> Self {
        self.append_condition("name!=", name.as_ref())
    }

    pub fn age_eq(self, age: VoiceAge) -> Self {
        self.append_condition("age=", age.into())
    }

    pub fn age_ne(self, age: VoiceAge) -> Self {
        self.append_condition("age!=", age.into())
    }

    pub fn gender_eq(self, gender: VoiceGender) -> Self {
        self.append_condition("gender=", gender.into())
    }

    pub fn gender_ne(self, gender: VoiceGender) -> Self {
        self.append_condition("gender!=", gender.into())
    }

    pub fn language_eq<S: AsRef<str>>(self, language: S) -> Self {
        self.append_condition("language=", language.as_ref())
    }

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

pub fn installed_voices(
    required: VoiceSelector,
    optional: Option<VoiceSelector>,
) -> Result<impl Iterator<Item = Voice>> {
    let category = Category::new(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Speech\Voices")?;
    let tokens = category
        .enum_tokens(required.into_sapi_expr(), optional.map(VoiceSelector::into_sapi_expr))?;

    Ok(tokens.map(|token| Voice {
        token,
    }))
}
