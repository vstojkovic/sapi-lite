use std::time::Duration;

use windows as Windows;
use Windows::Win32::Media::Speech::{
    ISpRecoContext, ISpRecognizer, SpInprocRecognizer, SPRST_ACTIVE, SPRST_INACTIVE,
};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::com_util::next_elem;
use crate::event::Event;
use crate::token::Category;
use crate::Result;

mod grammar;
mod phrase;

pub use grammar::{Grammar, GrammarBuilder, Rule};
pub use phrase::Phrase;

pub struct Recognizer {
    intf: ISpRecoContext,
}

impl Recognizer {
    pub fn new() -> Result<Self> {
        let recog: ISpRecognizer =
            unsafe { CoCreateInstance(&SpInprocRecognizer, None, CLSCTX_ALL) }?;

        let category = Category::new(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Speech\AudioInput")?;
        let token = category.default_token()?;
        unsafe { recog.SetInput(token.intf, false) }?;

        let intf = unsafe { recog.CreateRecoContext() }?;
        Ok(Recognizer {
            intf,
        })
    }

    pub fn set_state(&self, active: bool) -> Result<()> {
        let state = if active {
            SPRST_ACTIVE
        } else {
            SPRST_INACTIVE
        };
        self.recog_intf().and_then(|recog| unsafe { recog.SetRecoState(state) })
    }

    pub fn grammar_builder(&self) -> GrammarBuilder {
        GrammarBuilder::from_sapi(self.intf.clone())
    }

    pub fn recognize(&self, timeout: Duration) -> Result<Option<Phrase>> {
        let timeout_ms: u32 = timeout.as_millis().try_into().unwrap_or(u32::MAX - 1);
        unsafe { self.intf.WaitForNotifyEvent(timeout_ms) }?;

        while let Some(event) = unsafe { next_elem(&self.intf, ISpRecoContext::GetEvents) }? {
            let event = Event::from_sapi(event)?;
            if let Event::Recognition(result) = event {
                let phrase = Phrase::from_sapi(result)?;
                return Ok(Some(phrase));
            }
        }

        return Ok(None);
    }

    fn recog_intf(&self) -> Result<ISpRecognizer> {
        unsafe { self.intf.GetRecognizer() }
    }
}

impl Drop for Recognizer {
    fn drop(&mut self) {
        // The following call is expected to succeed, but failure shouldn't cause panic
        let _ = self.set_state(false);
    }
}
