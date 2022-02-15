use std::borrow::{Borrow, Cow};

use windows as Windows;
use Windows::Win32::Media::Speech::{SPF_DEFAULT, SPF_IS_XML, SPF_PARSE_SAPI};

mod builder;
mod types;

pub use builder::SpeechBuilder;
pub use types::{Pitch, Rate, SayAs, Volume};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Speech<'s> {
    Text(Cow<'s, str>),
    Xml(Cow<'s, str>),
}

impl<'s> Speech<'s> {
    pub(crate) fn flags(&self) -> u32 {
        (match self {
            Self::Text(_) => SPF_DEFAULT.0,
            Self::Xml(_) => (SPF_IS_XML.0 | SPF_PARSE_SAPI.0),
        }) as u32
    }

    pub(crate) fn contents(&self) -> &str {
        match self {
            Self::Text(cow) => cow.borrow(),
            Self::Xml(cow) => cow.borrow(),
        }
    }
}

impl<'s> From<&'s str> for Speech<'s> {
    fn from(s: &'s str) -> Self {
        Self::Text(s.into())
    }
}

impl<'s> From<String> for Speech<'s> {
    fn from(s: String) -> Self {
        Self::Text(s.into())
    }
}
