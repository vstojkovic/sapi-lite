#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! A simplified wrapper around Microsoft's Speech API (SAPI) library.
//!
//! # Features
//!
//! The goal of this crate is to expose a subset of SAPI features in a way that is easy to use in
//! Rust. It does not aim to provide the full set of features SAPI offers. Since the focus is on
//! relative ease of use (compared to dealing with COM directly) and simplicity of API, many
//! SAPI features are missing in this crate.
//!
//! ## Text-to-speech
//!
//! The [tts] module provides the API to render text as a speech using one or more SAPI voices
//! installed on the system.
//!
//! To generate speech, you first need to create an instance of one of the
//! available synthesizer types. Which synthesizer you choose will depend on whether you want to
//! block the execution while the speech is synthesized or not.
//!
//! You can configure the synthesizer to output to the default audio device, to a file, or to a
//! memory buffer. You can control the synthesizer's default speaking voice, rate of speech, and
//! volume.
//!
//! The speech you synthesize can be a simple string, or it can be a series of instructions
//! controlling various aspects of the speech synthesis, such as voice, rate, volume, pitch, or
//! pronunciation.
//!
//! You can enumerate the installed voices and filter them by one or more of their characteristics
//! (e.g. age or gender).
//!
//! ## Speech recognition
//!
//! The [stt] module provides the API to recognize speech, convert it to text, and annotate it with
//! semantic tags.
//!
//! The first thing you need is an instance of the recognition engine. Once created, you can
//! configure it to listen to the default recording device, to read a file, or to read from a
//! memory buffer. You can stop and resume recognition by disabling and enabling the recognizer.
//!
//! To configure which spoken phrases the engine can recognize, you need to define one or more
//! grammars. A grammar is a set of rules that define what word structures form recognizable
//! phrases.
//!
//! A grammar must be loaded into a recognition context before the engine can recognize the phrases
//! in it. Which context type you choose will depend on whether you want to block the execution
//! while waiting for a phrase to be recognized or not.
//!
//! # COM and Lifetime of SAPI Types
//!
//! Microsoft SAPI is a COM library. All COM objects and interfaces use reference counting to
//! control their lifetime. Many types in this crate wrap these COM objects. As such, when you drop
//! an instance of one of these types, it doesn't mean that the underlying COM object will be
//! destroyed.
//!
//! For example, if you have a [`Recognizer`](stt::Recognizer), a [`SyncContext`](stt::SyncContext),
//! and a [`Grammar`](stt::Grammar), dropping the `Recognizer` while the `SyncContext` or the
//! `Grammar` are still alive will *not* destroy the underlying recognition engine with its
//! associated contexts and rules.
//!
//! This crate does not currently represent these COM references using Rust lifetimes. This was a
//! deliberate design decision to keep the API and the code as simple as possible.

use std::ptr::null;

use windows::Win32::System::Com::{CoInitialize, CoUninitialize};

pub mod audio;
mod com_util;
mod event;
pub mod stt;
mod token;
pub mod tts;

#[cfg(feature = "tokio")]
pub mod tokio;

/// The error type returned by SAPI functions and methods.
pub type Error = windows::core::Error;

/// The type returned by SAPI functions and methods.
pub type Result<T> = windows::core::Result<T>;

/// Initializes SAPI on the current thread. This function must be called for every thread that
/// intends to use SAPI.
pub fn initialize() -> Result<()> {
    unsafe { CoInitialize(null()) }
}

/// Deinitializes SAPI for the current thread. This function must be called for every thread that
/// called `initialize()`, the same number of times.
pub fn finalize() {
    unsafe { CoUninitialize() }
}
