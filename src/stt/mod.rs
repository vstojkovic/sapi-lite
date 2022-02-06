use windows as Windows;
use Windows::Win32::Media::Speech::{
    ISpRecognizer, SpInprocRecognizer, SPRST_ACTIVE, SPRST_INACTIVE,
};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::token::Category;
use crate::Result;

mod context;
mod grammar;
mod phrase;
mod semantics;

#[cfg(feature = "tokio")]
pub mod tokio;

pub use context::{Context, EventHandler, EventfulContext, SyncContext};
pub use grammar::{Grammar, GrammarBuilder, Rule};
pub use phrase::Phrase;
pub use semantics::{SemanticString, SemanticTree, SemanticValue};

pub struct Recognizer {
    intf: ISpRecognizer,
}

impl Recognizer {
    pub fn new() -> Result<Self> {
        let intf: ISpRecognizer =
            unsafe { CoCreateInstance(&SpInprocRecognizer, None, CLSCTX_ALL) }?;

        let category = Category::new(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Speech\AudioInput")?;
        let token = category.default_token()?;
        unsafe { intf.SetInput(token.intf, false) }?;
        Ok(Self {
            intf,
        })
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let state = if enabled {
            SPRST_ACTIVE
        } else {
            SPRST_INACTIVE
        };
        unsafe { self.intf.SetRecoState(state) }
    }
}

impl Drop for Recognizer {
    fn drop(&mut self) {
        // The following call is expected to succeed, but failure shouldn't cause panic
        let _ = self.set_enabled(false);
    }
}
