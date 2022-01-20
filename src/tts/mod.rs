use std::mem::MaybeUninit;

use windows as Windows;
use Windows::Win32::Media::Speech::{ISpVoice, SpVoice};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::token::Token;
use crate::Result;

pub use self::speech::{Pitch, Rate, SayAs, Speech, SpeechBuilder, Volume};
pub use self::voice::{installed_voices, Voice, VoiceAge, VoiceGender, VoiceSelector};

mod speech;
mod voice;

pub struct Synthesizer {
    intf: ISpVoice,
}

impl Synthesizer {
    pub fn new() -> Result<Self> {
        unsafe { CoCreateInstance(&SpVoice, None, CLSCTX_ALL) }.map(|intf| Self {
            intf,
        })
    }

    pub fn rate(&self) -> Result<Rate> {
        let mut rate = MaybeUninit::uninit();
        unsafe { self.intf.GetRate(rate.as_mut_ptr()) }?;
        Ok(unsafe { rate.assume_init() }.into())
    }

    pub fn voice(&self) -> Result<Voice> {
        unsafe { self.intf.GetVoice() }.map(|intf| Voice {
            token: Token {
                intf,
            },
        })
    }

    pub fn volume(&self) -> Result<Volume> {
        let mut volume = MaybeUninit::<u16>::uninit();
        unsafe { self.intf.GetVolume(volume.as_mut_ptr()) }?;
        Ok(Volume::from_sapi(unsafe { volume.assume_init() }))
    }

    pub fn set_rate<R: Into<Rate>>(&self, rate: R) -> Result<()> {
        unsafe { self.intf.SetRate(rate.into().value()) }
    }

    pub fn set_voice(&self, voice: Voice) -> Result<()> {
        unsafe { self.intf.SetVoice(voice.token.intf) }
    }

    pub fn set_volume<V: Into<Volume>>(&self, volume: V) -> Result<()> {
        unsafe { self.intf.SetVolume(volume.into().sapi_value()) }
    }

    pub fn speak<'s, S: Into<Speech<'s>>>(&self, speech: S) -> Result<()> {
        let speech = speech.into();
        unsafe { self.intf.Speak(speech.contents(), speech.flags()) }.map(|_| ())
    }
}
