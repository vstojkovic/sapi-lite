use std::ops::Deref;
use std::time::Duration;

use windows as Windows;
use Windows::Win32::Media::Speech::SPF_ASYNC;
use Windows::Win32::System::WindowsProgramming::INFINITE;

use crate::tts::Speech;
use crate::Result;

use super::Synthesizer;

/// A speech synthesizer that blocks the current thread while rendering speech.
pub struct SyncSynthesizer {
    base: Synthesizer,
}

impl SyncSynthesizer {
    /// Creates a new synthesizer, configured to output its speech to the default audio device.
    pub fn new() -> Result<Self> {
        Ok(Self {
            base: Synthesizer::new()?,
        })
    }

    /// Renders the given speech, blocking the thread until done or until the given timeout expires.
    pub fn speak<'s, S: Into<Speech<'s>>>(
        &self,
        speech: S,
        timeout: Option<Duration>,
    ) -> Result<()> {
        self.base.speak(speech, SPF_ASYNC.0 as _)?;
        unsafe {
            self.base
                .intf
                .WaitUntilDone(timeout.map(|dur| dur.as_millis() as u32).unwrap_or(INFINITE))
        }
    }
}

impl Deref for SyncSynthesizer {
    type Target = Synthesizer;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
