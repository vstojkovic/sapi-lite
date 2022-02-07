use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::ops::{RangeInclusive, RangeToInclusive};
use std::ptr::{null, null_mut};

use windows as Windows;
use Windows::Win32::Media::Speech::{
    ISpRecoContext, ISpRecoGrammar, SPRAF_Active, SPRAF_TopLevel, SPGRAMMARSTATE, SPGS_DISABLED,
    SPGS_ENABLED, SPRS_ACTIVE, SPRS_INACTIVE, SPRULESTATE, SPSTATEHANDLE__, SPWT_LEXICAL,
};

use crate::com_util::{opt_str_param, out_to_ret, Intf};
use crate::Result;

use super::semantics::SemanticProperty;
use super::{RecognitionPauser, SemanticValue};

pub struct Grammar {
    intf: ManuallyDrop<Intf<ISpRecoGrammar>>,
    pauser: RecognitionPauser,
}

impl Grammar {
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let _pause = self.pauser.pause()?;
        unsafe { self.intf.SetGrammarState(grammar_state(enabled)) }
    }

    pub fn set_rule_enabled<S: AsRef<str>>(&self, name: S, enabled: bool) -> Result<()> {
        let _pause = self.pauser.pause()?;
        unsafe { self.intf.SetRuleState(name.as_ref(), null_mut(), rule_state(enabled)) }
    }
}

impl Drop for Grammar {
    fn drop(&mut self) {
        let _pause = self.pauser.pause();
        unsafe { ManuallyDrop::drop(&mut self.intf) };
    }
}

fn grammar_state(enabled: bool) -> SPGRAMMARSTATE {
    if enabled {
        SPGS_ENABLED
    } else {
        SPGS_DISABLED
    }
}

fn rule_state(enabled: bool) -> SPRULESTATE {
    if enabled {
        SPRS_ACTIVE
    } else {
        SPRS_INACTIVE
    }
}

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

pub struct GrammarBuilder<'a> {
    intf: Intf<ISpRecoContext>,
    pauser: RecognitionPauser,
    top_rules: HashSet<RuleRef<'a>>,
    rule_names: HashMap<RuleRef<'a>, Cow<'a, str>>,
}

impl<'a> GrammarBuilder<'a> {
    pub(super) fn new(intf: ISpRecoContext, pauser: RecognitionPauser) -> Self {
        Self {
            intf: Intf(intf),
            pauser,
            top_rules: HashSet::new(),
            rule_names: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: &'a Rule<'a>) -> &mut Self {
        self.top_rules.insert(RuleRef(rule));
        self
    }

    pub fn add_named_rule<S: Into<Cow<'a, str>>>(
        &mut self,
        name: S,
        rule: &'a Rule<'a>,
    ) -> &mut Self {
        self.add_rule(rule);
        self.rule_names.insert(RuleRef(rule), name.into());
        self
    }

    pub fn build(&mut self) -> Result<Grammar> {
        let grammar = unsafe { self.intf.CreateGrammar(0) }?;
        let mut rule_builder = RecursiveRuleBuilder {
            intf: grammar.clone(),
            owner: &self,
            built_rules: HashMap::new(),
        };
        for rule in self.top_rules.iter() {
            rule_builder.build_rule(rule.0)?;
        }
        unsafe { grammar.Commit(0) }?;
        unsafe { grammar.SetGrammarState(grammar_state(false)) }?;
        unsafe { grammar.SetRuleState(None, null_mut(), rule_state(true)) }?;
        Ok(Grammar {
            intf: ManuallyDrop::new(Intf(grammar)),
            pauser: self.pauser.clone(),
        })
    }
}

#[derive(Clone, Copy)]
struct RuleRef<'a>(&'a Rule<'a>);

impl<'a> PartialEq for RuleRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a> Eq for RuleRef<'a> {}

impl<'a> Hash for RuleRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state)
    }
}

type State = *mut SPSTATEHANDLE__;

struct RecursiveRuleBuilder<'a, 'b> {
    intf: ISpRecoGrammar,
    owner: &'b GrammarBuilder<'a>,
    built_rules: HashMap<RuleRef<'a>, State>,
}

impl<'a, 'b> RecursiveRuleBuilder<'a, 'b> {
    fn build_rule(&mut self, rule: &'a Rule<'a>) -> Result<State> {
        let rule_ref = RuleRef(rule);
        if let Some(state) = self.built_rules.get(&rule_ref) {
            return Ok(*state);
        }

        let flags = if self.owner.top_rules.contains(&rule_ref) {
            (SPRAF_TopLevel.0 | SPRAF_Active.0) as u32
        } else {
            0
        };
        let id: u32 = (self.built_rules.len() + 1).try_into().unwrap();
        let init_state = unsafe {
            out_to_ret(|out| {
                self.intf.GetRule(
                    opt_str_param(self.owner.rule_names.get(&rule_ref)).abi(),
                    id,
                    flags,
                    true,
                    out,
                )
            })
        }?;

        self.built_rules.insert(rule_ref, init_state);

        match rule {
            Rule::Text(text) => self.build_text(init_state, text)?,
            Rule::Choice(options) => self.build_choice(init_state, options)?,
            Rule::Sequence(parts) => self.build_sequence(init_state, parts)?,
            Rule::Repeat(times, target) => self.build_repeat(init_state, times, target)?,
            Rule::Semantic(sem_val, target) => self.build_semantic(init_state, sem_val, target)?,
        }

        Ok(init_state)
    }

    fn build_text(&mut self, init_state: State, text: &Cow<'a, str>) -> Result<()> {
        self.text_arc(init_state, null_mut(), text.as_ref())
    }

    fn build_choice(&mut self, init_state: State, options: &Cow<'a, [&Rule<'a>]>) -> Result<()> {
        for option in options.iter() {
            let child_state = self.build_rule(option)?;
            self.rule_arc(init_state, null_mut(), child_state, None)?;
        }
        Ok(())
    }

    fn build_sequence(&mut self, init_state: State, parts: &Cow<'a, [&'a Rule<'a>]>) -> Result<()> {
        let mut part_iter = parts.iter().peekable();
        let mut prev_state = init_state;
        while let Some(part) = part_iter.next() {
            let child_state = self.build_rule(part)?;
            let next_state = if part_iter.peek().is_some() {
                self.create_state(prev_state)?
            } else {
                null_mut()
            };
            self.rule_arc(prev_state, next_state, child_state, None)?;
            prev_state = next_state;
        }
        Ok(())
    }

    fn build_repeat(
        &mut self,
        init_state: State,
        times: &RepeatRange,
        target: &'a Rule<'a>,
    ) -> Result<()> {
        let child_state = self.build_rule(target)?;
        let mut prev_state = init_state;
        let mut occurences_left = times.max;
        let mut required_left = times.min;
        while occurences_left > 0 {
            occurences_left -= 1;
            let next_state = if occurences_left > 0 {
                self.create_state(prev_state)?
            } else {
                null_mut()
            };
            self.rule_arc(prev_state, next_state, child_state, None)?;
            if required_left > 0 {
                required_left -= 1;
            } else {
                self.epsilon_arc(prev_state, null_mut())?;
            }
            prev_state = next_state;
        }
        Ok(())
    }

    fn build_semantic(
        &mut self,
        init_state: State,
        sem_val: &SemanticValue<Cow<'a, str>>,
        target: &'a Rule<'a>,
    ) -> Result<()> {
        let child_state = self.build_rule(target)?;
        let property = SemanticProperty::new(sem_val);
        self.rule_arc(init_state, null_mut(), child_state, Some(&property))
    }

    fn create_state(&mut self, from_state: State) -> Result<State> {
        unsafe { out_to_ret(|out| self.intf.CreateNewState(from_state, out)) }
    }

    fn text_arc(&mut self, from_state: State, to_state: State, text: &str) -> Result<()> {
        unsafe {
            self.intf.AddWordTransition(from_state, to_state, text, " ", SPWT_LEXICAL, 1.0, null())
        }
    }

    fn rule_arc(
        &mut self,
        from_state: State,
        to_state: State,
        child_state: State,
        property: Option<&SemanticProperty>,
    ) -> Result<()> {
        let prop_ptr = match property {
            Some(prop) => &prop.info,
            None => null(),
        };
        unsafe { self.intf.AddRuleTransition(from_state, to_state, child_state, 1.0, prop_ptr) }
    }

    fn epsilon_arc(&mut self, from_state: State, to_state: State) -> Result<()> {
        unsafe {
            self.intf.AddWordTransition(from_state, to_state, None, None, SPWT_LEXICAL, 1.0, null())
        }
    }
}
