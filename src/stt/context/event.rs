use std::ops::Deref;

use windows as Windows;
use Windows::core::Interface;

use crate::event::{Event, EventSink, EventSource};
use crate::stt::{Phrase, Recognizer};
use crate::Result;

use super::Context;

/// The handler [`EventfulContext`] will call.
pub trait EventHandler: Sync {
    /// Called when the engine has successfully recognized a phrase.
    fn on_recognition(&self, phrase: Phrase);
}

impl<F: Fn(Phrase) + Sync> EventHandler for F {
    fn on_recognition(&self, phrase: Phrase) {
        self(phrase)
    }
}

/// A recognition context that calls the supplied event handler every time the engine recognizes a
/// phrase in it.
pub struct EventfulContext {
    base: Context,
}

impl EventfulContext {
    /// Creates a new recognition context for the given recognizer, configured to call the given
    /// handler whenever a phrase from this context is recognized.
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
