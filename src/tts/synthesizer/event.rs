use std::ops::Deref;

use windows as Windows;
use Windows::core::Interface;
use Windows::Win32::Media::Speech::{SPEI_END_INPUT_STREAM, SPF_ASYNC};

use crate::event::{Event, EventSink, EventSource};
use crate::tts::Speech;
use crate::Result;

use super::Synthesizer;

/// The handler [`EventfulSynthesizer`] will call.
pub trait EventHandler: Sync {
    /// Called when the synthesizer has finished rendering the speech with the given identifier.
    fn on_speech_finished(&self, id: u32);
}

impl<F: Fn(u32) + Sync> EventHandler for F {
    fn on_speech_finished(&self, id: u32) {
        self(id)
    }
}

/// A speech synthesizer that calls the supplied event handler every time it finishes rendering
/// speech.
pub struct EventfulSynthesizer {
    base: Synthesizer,
}

impl EventfulSynthesizer {
    /// Creates a new synthesizer that will output to the default audio device and call the supplied
    /// event handler.
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

    /// Schedules the rendering of the given speech and returns a numeric identifier for it. The
    /// identifier will be passed to the event handler when the speech has been rendered.
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
