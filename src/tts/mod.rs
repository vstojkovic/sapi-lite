use std::mem::MaybeUninit;

use windows as Windows;
use Windows::Win32::Media::Speech::{ISpVoice, SpVoice};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::token::Token;
use crate::Result;

pub use self::speech::{SayAs, Speech, SpeechBuilder};
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

    pub fn rate(&self) -> Result<i32> {
        let mut rate = MaybeUninit::uninit();
        unsafe { self.intf.GetRate(rate.as_mut_ptr()) }?;
        Ok(unsafe { rate.assume_init() })
    }

    pub fn voice(&self) -> Result<Voice> {
        unsafe { self.intf.GetVoice() }.map(|intf| Voice {
            token: Token {
                intf,
            },
        })
    }

    pub fn volume(&self) -> Result<i32> {
        let mut volume = MaybeUninit::<u16>::uninit();
        unsafe { self.intf.GetVolume(volume.as_mut_ptr()) }?;
        Ok(unsafe { volume.assume_init().into() })
    }

    pub fn set_rate(&self, rate: i32) -> Result<()> {
        unsafe { self.intf.SetRate(rate.clamp(-10, 10)) }
    }

    pub fn set_voice(&self, voice: Voice) -> Result<()> {
        unsafe { self.intf.SetVoice(voice.token.intf) }
    }

    pub fn set_volume(&self, volume: i32) -> Result<()> {
        unsafe { self.intf.SetVolume(volume.clamp(0, 100) as _) }
    }

    pub fn speak<'s, S: Into<Speech<'s>>>(&self, speech: S) -> Result<()> {
        let speech = speech.into();
        unsafe { self.intf.Speak(speech.contents(), speech.flags()) }.map(|_| ())
    }
}
