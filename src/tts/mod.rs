//! Text-to-speech API.
//!
//! ## Synthesizer
//!
//! The entry point for speech synthesis is the synthesizer. This module provides two variants of
//! synthesizers:
//! * [`SyncSynthesizer`] will block the current thread until the speech synthesis is finished, or
//! until the given timeout.
//! * [`EventfulSynthesizer`] will return immediately and call the supplied event handler when the
//! speech is finished.
//!
//! For asynchronous synthesis, see the [`tokio`](crate::tokio) module.
//!
//! All synthesizers share the methods defined in the [`Synthesizer`] struct.
//!
//! ## Voice
//!
//! The user can install a variety of voices on their machine. The [`installed_voices`] function
//! allows iterating through all the installed voices, filtered by the provided criteria.

mod speech;
mod synthesizer;
mod voice;

pub use self::speech::{Pitch, Rate, SayAs, Speech, SpeechBuilder, Volume};
pub use self::synthesizer::{
    EventHandler, EventfulSynthesizer, SpeechOutput, SyncSynthesizer, Synthesizer,
};
pub use self::voice::{installed_voices, Voice, VoiceAge, VoiceGender, VoiceSelector};
