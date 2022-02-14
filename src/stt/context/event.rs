use std::ops::Deref;

use windows as Windows;
use Windows::core::Interface;

use crate::event::{Event, EventSink, EventSource};
use crate::stt::{Phrase, Recognizer};
use crate::Result;

use super::Context;

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
