use std::ops::Deref;
use std::time::Duration;

use windows as Windows;
use Windows::core::Interface;

use crate::event::{Event, EventSource};
use crate::stt::{Phrase, Recognizer};
use crate::Result;

use super::Context;

pub struct SyncContext {
    base: Context,
    event_src: EventSource,
}

impl SyncContext {
    pub fn new(recognizer: &Recognizer) -> Result<Self> {
        let intf = unsafe { recognizer.intf.CreateRecoContext() }?;
        unsafe { intf.SetNotifyWin32Event() }?;
        Ok(SyncContext {
            event_src: EventSource::from_sapi(intf.cast()?),
            base: Context::new(intf, recognizer.pauser.clone()),
        })
    }

    pub fn recognize(&self, timeout: Duration) -> Result<Option<Phrase>> {
        let result = self.next_phrase()?;
        if result.is_some() {
            return Ok(result);
        }

        let timeout_ms: u32 = timeout.as_millis().try_into().unwrap_or(u32::MAX - 1);
        unsafe { self.base.intf.WaitForNotifyEvent(timeout_ms) }?;

        return self.next_phrase();
    }

    fn next_phrase(&self) -> Result<Option<Phrase>> {
        while let Some(event) = self.event_src.next_event()? {
            if let Event::Recognition(result) = event {
                let phrase = Phrase::from_sapi(result)?;
                return Ok(Some(phrase));
            }
        }
        Ok(None)
    }
}

impl Deref for SyncContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
