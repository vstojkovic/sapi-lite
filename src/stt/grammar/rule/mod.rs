use std::borrow::Cow;
use std::ops::{RangeInclusive, RangeToInclusive};

use crate::stt::SemanticValue;

mod arena;

pub use arena::RuleArena;

/// A rule that defines one or more phrases or fragments that can be recognized by the engine.
#[derive(Debug)]
pub enum Rule<'a> {
    /// A sequence of words
    Text(Cow<'a, str>),
    /// A set of rules to choose from
    Choice(Cow<'a, [&'a Rule<'a>]>),
    /// A sequence of rules that must be recognized in order
    Sequence(Cow<'a, [&'a Rule<'a>]>),
    /// A rule repeated a certain number of times
    Repeat(RepeatRange, &'a Rule<'a>),
    /// A rule that will produce a node in the semantic tree when recognized
    Semantic(SemanticValue<Cow<'a, str>>, &'a Rule<'a>),
}

impl<'a> Rule<'a> {
    /// Creates a rule that defines a sequence of words to be recognized.
    pub fn text<T: Into<Cow<'a, str>>>(text: T) -> Self {
        Self::Text(text.into())
    }

    /// Creates a rule that defines a set of alternatives to choose from.
    pub fn choice<L: Into<Cow<'a, [&'a Rule<'a>]>>>(options: L) -> Self {
        Self::Choice(options.into())
    }

    /// Creates a rule the defines a sequence of sub-rules that must be recognized in order.
    pub fn sequence<L: Into<Cow<'a, [&'a Rule<'a>]>>>(parts: L) -> Self {
        Self::Sequence(parts.into())
    }

    /// Creates a rule that recognizes a sub-rule repeated a certain number of times.
    pub fn repeat<R: Into<RepeatRange>>(times: R, target: &'a Rule<'a>) -> Self {
        Self::Repeat(times.into(), target)
    }

    /// Creates a rule that produces a node in the resulting semantic tree when the given sub-rule
    /// is recognized.
    pub fn semantic<V: Into<SemanticValue<Cow<'a, str>>>>(value: V, target: &'a Rule<'a>) -> Self {
        Self::Semantic(value.into(), target)
    }
}

/// Specifies the bounds for how many times the target rule in a [`Rule::Repeat`] can be repeated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatRange {
    /// The target rule must be repeated at least this many times.
    pub min: usize,
    /// The target rule can be repeated at most this many times.
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
