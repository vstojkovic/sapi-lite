use std::ops::Deref;

use windows as Windows;
use Windows::core::Interface;
use Windows::Win32::Media::Speech::{SPEI_END_INPUT_STREAM, SPF_ASYNC};

use crate::event::{Event, EventSink, EventSource};
use crate::tts::Speech;
use crate::Result;

use super::Synthesizer;

pub trait EventHandler: Sync {
    fn on_speech_finished(&self, id: u32);
}

impl<F: Fn(u32) + Sync> EventHandler for F {
    fn on_speech_finished(&self, id: u32) {
        self(id)
    }
}

pub struct EventfulSynthesizer {
    base: Synthesizer,
}

impl EventfulSynthesizer {
    pub fn new<E: EventHandler + 'static>(handler: E) -> Result<Self> {
        let base = Synthesizer::new()?;
        EventSink::new(EventSource::from_sapi(base.intf.0.cast()?), move |event| {
            if let Event::SpeechFinished(id) = event {
                handler.on_speech_finished(id);
            }
            Ok(())
        })
        .install(Some(&[SPEI_END_INPUT_STREAM]))?;
        Ok(Self {
            base,
        })
    }

    pub fn speak<'s, S: Into<Speech<'s>>>(&self, speech: S) -> Result<u32> {
        self.base.speak(speech, SPF_ASYNC.0 as _)
    }
}

impl Deref for EventfulSynthesizer {
    type Target = Synthesizer;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
