use std::borrow::Cow;
use std::ops::{RangeInclusive, RangeToInclusive};

use crate::stt::SemanticValue;

#[derive(Debug)]
pub enum Rule<'a> {
    Text(Cow<'a, str>),
    Choice(Cow<'a, [&'a Rule<'a>]>),
    Sequence(Cow<'a, [&'a Rule<'a>]>),
    Repeat(RepeatRange, &'a Rule<'a>),
    Semantic(SemanticValue<Cow<'a, str>>, &'a Rule<'a>),
}

impl<'a> Rule<'a> {
    pub fn text<T: Into<Cow<'a, str>>>(text: T) -> Self {
        Self::Text(text.into())
    }

    pub fn choice<L: Into<Cow<'a, [&'a Rule<'a>]>>>(options: L) -> Self {
        Self::Choice(options.into())
    }

    pub fn sequence<L: Into<Cow<'a, [&'a Rule<'a>]>>>(parts: L) -> Self {
        Self::Sequence(parts.into())
    }

    pub fn repeat<R: Into<RepeatRange>>(times: R, target: &'a Rule<'a>) -> Self {
        Self::Repeat(times.into(), target)
    }

    pub fn semantic<V: Into<SemanticValue<Cow<'a, str>>>>(value: V, target: &'a Rule<'a>) -> Self {
        Self::Semantic(value.into(), target)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatRange {
    pub min: usize,
    pub max: usize,
}

impl From<usize> for RepeatRange {
    fn from(source: usize) -> Self {
        Self {
            min: source,
            max: source,
        }
    }
}

impl From<RangeInclusive<usize>> for RepeatRange {
    fn from(source: RangeInclusive<usize>) -> Self {
        Self {
            min: *source.start(),
            max: *source.end(),
        }
    }
}

impl From<RangeToInclusive<usize>> for RepeatRange {
    fn from(source: RangeToInclusive<usize>) -> Self {
        Self {
            min: 0,
            max: source.end,
        }
    }
}
