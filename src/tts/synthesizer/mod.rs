use windows as Windows;
use Windows::core::IUnknown;
use Windows::Win32::Media::Speech::{ISpVoice, SpVoice};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::audio::AudioStream;
use crate::com_util::{out_to_ret, Intf};
use crate::token::Token;
use crate::Result;

use super::{Rate, Speech, Voice, Volume};

mod event;
mod sync;

pub use event::{EventHandler, EventfulSynthesizer};
pub use sync::SyncSynthesizer;

/// Specifies where the output of speech synthesis should go.
pub enum SpeechOutput {
    /// Output to the default audio device on the system
    Default,
    /// Write to the given stream
    Stream(AudioStream),
}

impl SpeechOutput {
    fn to_sapi(self) -> Option<IUnknown> {
        match self {
            Self::Default => None,
            Self::Stream(stream) => Some(stream.to_sapi().0),
        }
    }
}

/// Provides the common speech synthesis API shared across different kinds of synthesizers.
pub struct Synthesizer {
    intf: Intf<ISpVoice>,
}

impl Synthesizer {
    fn new() -> Result<Self> {
        unsafe { CoCreateInstance(&SpVoice, None, CLSCTX_ALL) }.map(|intf| Self {
            intf: Intf(intf),
        })
    }

    /// Configures the synthesizer to render its speech to the given output destination.
    pub fn set_output(&self, output: SpeechOutput, allow_fmt_changes: bool) -> Result<()> {
        unsafe { self.intf.SetOutput(output.to_sapi(), allow_fmt_changes) }
    }

    /// Returns the default rate of speech for this synthesizer.
    pub fn rate(&self) -> Result<Rate> {
        unsafe { out_to_ret(|out| self.intf.GetRate(out)) }.map(Rate::new)
    }

    /// Returns the default voice this synthesizer will use to render speech.
    pub fn voice(&self) -> Result<Voice> {
        unsafe { self.intf.GetVoice() }.map(|intf| Voice {
            token: Token::from_sapi(intf),
        })
    }

    /// Returns the default speech volume for this synthesizer.
    pub fn volume(&self) -> Result<Volume> {
        unsafe { out_to_ret(|out| self.intf.GetVolume(out)) }.map(Volume::from_sapi)
    }

    /// Sets the default rate of speech for this synthesizer.
    pub fn set_rate<R: Into<Rate>>(&self, rate: R) -> Result<()> {
        unsafe { self.intf.SetRate(rate.into().value()) }
    }

    /// Sets the default voice this synthesizer will use to render speech.
    pub fn set_voice(&self, voice: Voice) -> Result<()> {
        unsafe { self.intf.SetVoice(voice.token) }
    }

    /// Sets the default speech volume for this synthesizer.
    pub fn set_volume<V: Into<Volume>>(&self, volume: V) -> Result<()> {
        unsafe { self.intf.SetVolume(volume.into().sapi_value()) }
    }

    fn speak<'s, S: Into<Speech<'s>>>(&self, speech: S, base_flags: u32) -> Result<u32> {
        let speech = speech.into();
        unsafe { self.intf.Speak(speech.contents(), speech.flags() | base_flags) }
    }
}
