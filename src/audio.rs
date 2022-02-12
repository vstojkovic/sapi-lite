use std::path::Path;

use windows as Windows;
use Windows::core::GUID;
use Windows::Win32::Media::Audio::{WAVEFORMATEX, WAVE_FORMAT_PCM};
use Windows::Win32::Media::Speech::{
    ISpStream, SpStream, SPFILEMODE, SPFM_CREATE_ALWAYS, SPFM_OPEN_READONLY,
};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::com_util::Intf;
use crate::Result;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SampleRate {
    Hz8000 = 8000,
    Hz11025 = 11025,
    Hz12000 = 12000,
    Hz16000 = 16000,
    Hz22050 = 22050,
    Hz24000 = 24000,
    Hz32000 = 32000,
    Hz44100 = 44100,
    Hz48000 = 48000,
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum BitRate {
    Bits8 = 8,
    Bits16 = 16,
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Channels {
    Mono = 1,
    Stereo = 2,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct AudioFormat {
    pub sample_rate: SampleRate,
    pub bit_rate: BitRate,
    pub channels: Channels,
}

impl AudioFormat {
    fn to_sapi(&self) -> WAVEFORMATEX {
        let block_align = (self.channels as u32) * (self.bit_rate as u32) / 8;
        WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_PCM as _,
            nChannels: self.channels as u16,
            nSamplesPerSec: self.sample_rate as u32,
            nAvgBytesPerSec: (self.sample_rate as u32) * block_align,
            nBlockAlign: block_align as u16,
            wBitsPerSample: self.bit_rate as u16,
            cbSize: 0,
        }
    }
}

pub struct AudioStream {
    intf: Intf<ISpStream>,
}

#[allow(non_upper_case_globals)]
const SPDFID_WaveFormatEx: GUID = GUID::from_u128(0xc31adbae_527f_4ff5_a230_f62bb61ff70c);

impl AudioStream {
    pub fn open_file<P: AsRef<Path>>(path: P, format: &AudioFormat) -> Result<Self> {
        Self::from_file(path, format, SPFM_OPEN_READONLY)
    }

    pub fn create_file<P: AsRef<Path>>(path: P, format: &AudioFormat) -> Result<Self> {
        Self::from_file(path, format, SPFM_CREATE_ALWAYS)
    }

    fn from_file<P: AsRef<Path>>(path: P, format: &AudioFormat, mode: SPFILEMODE) -> Result<Self> {
        let intf: ISpStream = unsafe { CoCreateInstance(&SpStream, None, CLSCTX_ALL) }?;
        unsafe {
            intf.BindToFile(
                path.as_ref().as_os_str(),
                mode,
                &SPDFID_WaveFormatEx,
                &format.to_sapi(),
                0,
            )
        }?;
        Ok(AudioStream {
            intf: Intf(intf),
        })
    }

    pub(crate) fn to_sapi(&self) -> ISpStream {
        self.intf.0.clone()
    }
}
