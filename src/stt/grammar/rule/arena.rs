use std::borrow::Cow;

use typed_arena::Arena;

use crate::stt::SemanticValue;

use super::{RepeatRange, Rule};

/// Allocation arena for grammar rules.
///
/// Provides an easy way to allocate [`Rule`] instances while building a grammar. All of the rules
/// owned by the arena will be dropped when the arena itself is dropped.
pub struct RuleArena<'a> {
    arena: Arena<Rule<'a>>,
}

impl<'a> RuleArena<'a> {
    /// Construct a new arena.
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    /// Take ownership of a constructed rule and keep it until the arena is destroyed.
    pub fn alloc(&self, rule: Rule<'a>) -> &Rule<'a> {
        self.arena.alloc(rule)
    }

    /// Allocate a rule that defines a sequence of words to be recognized.
    pub fn text<T: Into<Cow<'a, str>>>(&self, text: T) -> &Rule<'a> {
        self.alloc(Rule::text(text))
    }

    /// Allocate a rule that defines a set of alternatives to choose from.
    pub fn choice<L: Into<Cow<'a, [&'a Rule<'a>]>>>(&self, options: L) -> &Rule<'a> {
        self.alloc(Rule::choice(options))
    }

    /// Allocate a rule the defines a sequence of sub-rules that must be recognized in order.
    pub fn sequence<L: Into<Cow<'a, [&'a Rule<'a>]>>>(&self, parts: L) -> &Rule<'a> {
        self.alloc(Rule::sequence(parts))
    }

    /// Allocate a rule that recognizes a sub-rule repeated a certain number of times.
    pub fn repeat<R: Into<RepeatRange>>(&self, times: R, target: &'a Rule<'a>) -> &Rule<'a> {
        self.alloc(Rule::repeat(times, target))
    }

    /// Allocate a rule that produces a node in the resulting semantic tree when the given sub-rule
    /// is recognized.
    pub fn semantic<V: Into<SemanticValue<Cow<'a, str>>>>(
        &self,
        value: V,
        target: &'a Rule<'a>,
    ) -> &Rule<'a> {
        self.alloc(Rule::semantic(value, target))
    }
}
