use std::mem::ManuallyDrop;
use std::ptr::null_mut;

use windows as Windows;
use Windows::Win32::Media::Speech::{
    ISpRecoGrammar, SPGRAMMARSTATE, SPGS_DISABLED, SPGS_ENABLED, SPRS_ACTIVE, SPRS_INACTIVE,
    SPRULESTATE,
};

use crate::com_util::Intf;
use crate::Result;

use super::RecognitionPauser;

mod builder;
mod rule;

pub use builder::GrammarBuilder;
pub use rule::{RepeatRange, Rule};

/// A set of rules that define phrases that can be recognized.
pub struct Grammar {
    intf: ManuallyDrop<Intf<ISpRecoGrammar>>,
    pauser: RecognitionPauser,
}

impl Grammar {
    /// Enables or disables the recognition of all the phrases defined in this grammar.
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let _pause = self.pauser.pause()?;
        unsafe { self.intf.SetGrammarState(grammar_state(enabled)) }
    }

    /// Enables or disables the recognition of the phrases defined by the rule with the given name.
    pub fn set_rule_enabled<S: AsRef<str>>(&self, name: S, enabled: bool) -> Result<()> {
        let _pause = self.pauser.pause()?;
        unsafe {
            self.intf
                .SetRuleState(name.as_ref(), null_mut(), rule_state(enabled))
        }
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
