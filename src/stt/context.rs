use std::ops::Deref;
use std::time::Duration;

use windows as Windows;
use Windows::core::Interface;
use Windows::Win32::Media::Speech::{ISpRecoContext, SPCS_DISABLED, SPCS_ENABLED};

use crate::com_util::Intf;
use crate::event::{Event, EventSink, EventSource};
use crate::Result;

use super::{GrammarBuilder, Phrase, RecognitionPauser, Recognizer};

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

pub trait EventHandler: Sync {
    fn on_recognition(&self, phrase: Phrase);
}

impl<F: Fn(Phrase) + Sync> EventHandler for F {
    fn on_recognition(&self, phrase: Phrase) {
        self(phrase)
    }
}

pub struct EventfulContext {
    base: Context,
}

impl EventfulContext {
    pub fn new<E: EventHandler + 'static>(recognizer: &Recognizer, handler: E) -> Result<Self> {
        let intf = unsafe { recognizer.intf.CreateRecoContext() }?;
        EventSink::new(EventSource::from_sapi(intf.cast()?), move |event| {
            if let Event::Recognition(result) = event {
                let phrase = Phrase::from_sapi(result)?;
                handler.on_recognition(phrase);
            }
            Ok(())
        })
        .install(None)?;
        Ok(Self {
            base: Context::new(intf, recognizer.pauser.clone()),
        })
    }
}

impl Deref for EventfulContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
