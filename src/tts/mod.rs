use std::ops::Deref;
use std::time::Duration;

use windows as Windows;
use Windows::core::{IUnknown, Interface};
use Windows::Win32::Media::Speech::{ISpVoice, SpVoice, SPEI_END_INPUT_STREAM, SPF_ASYNC};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use Windows::Win32::System::WindowsProgramming::INFINITE;

use crate::audio::AudioStream;
use crate::com_util::{out_to_ret, Intf};
use crate::event::{Event, EventSink, EventSource};
use crate::token::Token;
use crate::Result;

mod speech;
mod voice;

pub use self::speech::{Pitch, Rate, SayAs, Speech, SpeechBuilder, Volume};
pub use self::voice::{installed_voices, Voice, VoiceAge, VoiceGender, VoiceSelector};

pub enum SpeechOutput {
    Default,
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

pub struct Synthesizer {
    intf: Intf<ISpVoice>,
}

impl Synthesizer {
    fn new() -> Result<Self> {
        unsafe { CoCreateInstance(&SpVoice, None, CLSCTX_ALL) }.map(|intf| Self {
            intf: Intf(intf),
        })
    }

    pub fn set_output(&self, output: SpeechOutput, allow_fmt_changes: bool) -> Result<()> {
        unsafe { self.intf.SetOutput(output.to_sapi(), allow_fmt_changes) }
    }

    pub fn rate(&self) -> Result<Rate> {
        unsafe { out_to_ret(|out| self.intf.GetRate(out)) }.map(Rate::new)
    }

    pub fn voice(&self) -> Result<Voice> {
        unsafe { self.intf.GetVoice() }.map(|intf| Voice {
            token: Token::from_sapi(intf),
        })
    }

    pub fn volume(&self) -> Result<Volume> {
        unsafe { out_to_ret(|out| self.intf.GetVolume(out)) }.map(Volume::from_sapi)
    }

    pub fn set_rate<R: Into<Rate>>(&self, rate: R) -> Result<()> {
        unsafe { self.intf.SetRate(rate.into().value()) }
    }

    pub fn set_voice(&self, voice: Voice) -> Result<()> {
        unsafe { self.intf.SetVoice(voice.token) }
    }

    pub fn set_volume<V: Into<Volume>>(&self, volume: V) -> Result<()> {
        unsafe { self.intf.SetVolume(volume.into().sapi_value()) }
    }

    fn speak<'s, S: Into<Speech<'s>>>(&self, speech: S, base_flags: u32) -> Result<u32> {
        let speech = speech.into();
        unsafe { self.intf.Speak(speech.contents(), speech.flags() | base_flags) }
    }
}

pub struct SyncSynthesizer {
    base: Synthesizer,
}

impl SyncSynthesizer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            base: Synthesizer::new()?,
        })
    }

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
