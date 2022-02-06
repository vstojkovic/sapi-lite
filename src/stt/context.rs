use std::ops::Deref;
use std::time::Duration;

use windows as Windows;
use Windows::core::implement;
use Windows::Win32::Media::Speech::{ISpNotifySink, ISpRecoContext, SPCS_DISABLED, SPCS_ENABLED};

use crate::com_util::next_elem;
use crate::event::Event;
use crate::Result;

use super::{GrammarBuilder, Phrase, RecognitionPauser, Recognizer};

pub struct Context {
    intf: ISpRecoContext,
    pauser: RecognitionPauser,
}

impl Context {
    fn new(intf: ISpRecoContext, pauser: RecognitionPauser) -> Self {
        Self {
            intf,
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

    fn next_event(&self) -> Result<Option<Event>> {
        next_ctx_event(&self.intf)
    }
}

fn next_ctx_event(intf: &ISpRecoContext) -> Result<Option<Event>> {
    Ok(match unsafe { next_elem(intf, ISpRecoContext::GetEvents) }? {
        Some(sapi_event) => Some(Event::from_sapi(sapi_event)?),
        None => None,
    })
}

pub struct SyncContext {
    base: Context,
}

impl SyncContext {
    pub fn new(recognizer: &Recognizer) -> Result<Self> {
        let intf = unsafe { recognizer.intf.CreateRecoContext() }?;
        unsafe { intf.SetNotifyWin32Event() }?;
        Ok(SyncContext {
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
        while let Some(event) = self.base.next_event()? {
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
        let sink: ISpNotifySink = EventSink::new(intf.clone(), handler).into();
        unsafe { intf.SetNotifySink(sink) }?;
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

#[implement(Windows::Win32::Media::Speech::ISpNotifySink)]
struct EventSink {
    intf: ISpRecoContext,
    handler: Box<dyn EventHandler>,
}

#[allow(non_snake_case)]
impl EventSink {
    fn new<E: EventHandler + 'static>(intf: ISpRecoContext, handler: E) -> Self {
        Self {
            intf,
            handler: Box::new(handler),
        }
    }

    fn Notify(&self) -> Result<()> {
        while let Some(event) = next_ctx_event(&self.intf)? {
            if let Event::Recognition(result) = event {
                let phrase = Phrase::from_sapi(result)?;
                self.handler.on_recognition(phrase);
            }
        }
        Ok(())
    }
}
