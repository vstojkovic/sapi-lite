use windows as Windows;
use Windows::Win32::Media::Speech::{ISpRecoContext, SPCS_DISABLED, SPCS_ENABLED};

use crate::com_util::Intf;
use crate::Result;

use super::{GrammarBuilder, RecognitionPauser};

mod event;
mod sync;

pub use event::{EventHandler, EventfulContext};
pub use sync::SyncContext;

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

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let state = if enabled {
            SPCS_ENABLED
        } else {
            SPCS_DISABLED
        };
        unsafe { self.intf.SetContextState(state) }
    }

    pub fn grammar_builder(&self) -> GrammarBuilder {
        GrammarBuilder::new(self.intf.clone(), self.pauser.clone())
    }
}
