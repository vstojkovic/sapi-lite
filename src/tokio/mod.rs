// #![cfg_attr(docsrs, doc(cfg(any(feature = "tokio-rt", feature = "tokio-stt", feature = "tokio-tts"))))]
//! Support for async operations running on Tokio.

#[cfg(feature = "tokio-rt")]
mod rt;
#[cfg(feature = "tokio-stt")]
mod stt;
#[cfg(feature = "tokio-tts")]
mod tts;

#[cfg(feature = "tokio-rt")]
pub use rt::BuilderExt;
#[cfg(feature = "tokio-stt")]
pub use stt::{
    BroadcastContext, BroadcastResult, BroadcastSubscriber, UnicastContext, UnicastSubscriber,
};
#[cfg(feature = "tokio-tts")]
pub use tts::AsyncSynthesizer;
