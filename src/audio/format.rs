use windows as Windows;
use Windows::Win32::Media::Audio::{WAVEFORMATEX, WAVE_FORMAT_PCM};

/// Sample rate, in samples per second, at which to play or record.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[allow(missing_docs)]
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

/// How many bits each sample should have.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[allow(missing_docs)]
pub enum BitRate {
    Bits8 = 8,
    Bits16 = 16,
}

/// Number of audio channels.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[allow(missing_docs)]
pub enum Channels {
    Mono = 1,
    Stereo = 2,
}

/// Specifies the format of the audio data in a stream.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct AudioFormat {
    /// Sample rate at which to play or record.
    pub sample_rate: SampleRate,
    /// How many bits each sample should have.
    pub bit_rate: BitRate,
    /// Number of channels.
    pub channels: Channels,
}

impl AudioFormat {
    pub(super) fn to_sapi(&self) -> WAVEFORMATEX {
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
