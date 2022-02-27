use windows as Windows;
use Windows::Win32::Media::Speech::{ISpRecoContext, SPCS_DISABLED, SPCS_ENABLED};

use crate::com_util::Intf;
use crate::Result;

use super::{GrammarBuilder, RecognitionPauser};

mod event;
mod sync;

pub use event::{EventHandler, EventfulContext};
pub use sync::SyncContext;

/// Provides the common API shared across different kinds of contexts.
pub struct Context {
    intf: Intf<ISpRecoContext>,
    pauser: RecognitionPauser,
}

impl Context {
    fn new(intf: ISpRecoContext, pauser: RecognitionPauser) -> Self {
        Self {
            intf: Intf(intf),
            pauser,
        }
    }

    /// Enables or disables the recognition of rules from all grammars loaded into this context.
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let state = if enabled { SPCS_ENABLED } else { SPCS_DISABLED };
        unsafe { self.intf.SetContextState(state) }
    }

    /// Creates a [`GrammarBuilder`] that will construct and load a grammar into this context.
    pub fn grammar_builder(&self) -> GrammarBuilder {
        GrammarBuilder::new(self.intf.clone(), self.pauser.clone())
    }
}
