//! Support for streaming audio to and from files and memory buffers.

mod format;
mod stream;

pub use format::{AudioFormat, BitRate, Channels, SampleRate};
pub use stream::{AudioStream, MemoryStream};
