//! Speech recognition API.
//!
//! ## Recognizer
//!
//! The entry point for speech recognition is the [`Recognizer`], which encapsulates an in-process
//! speech recognition engine. You generally won't need more than one instance of the recognizer.
//!
//! ## Context
//!
//! The recognizer can have one or more recognition contexts. This module provides two variants of
//! contexts:
//! * [`SyncContext`] will block the current thread until the engine recognizes a phrase, or until
//! the given timeout.
//! * [`EventfulContext`] will call the supplied event handler whenever the engine recognizes a
//! phrase.
//!
//! For asynchronous recognition, see the [`tokio`](crate::tokio) module.
//!
//! ## Grammar
//!
//! Each context can have one or more grammars loaded into it. A grammar consists of one or more
//! rules that define what phrases the engine can recognize. You can enable or disable the whole
//! grammar, or individual rules in it by their name.

use std::sync::{Arc, Mutex};

use windows as Windows;
use Windows::core::IUnknown;
use Windows::Win32::Media::Speech::{
    ISpRecognizer, SpInprocRecognizer, SPRECOSTATE, SPRST_ACTIVE, SPRST_INACTIVE,
};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::audio::AudioStream;
use crate::com_util::Intf;
use crate::token::Category;
use crate::Result;

mod context;
mod grammar;
mod phrase;
mod semantics;

pub use context::{Context, EventHandler, EventfulContext, SyncContext};
pub use grammar::{Grammar, GrammarBuilder, RepeatRange, Rule};
pub use phrase::Phrase;
pub use semantics::{SemanticString, SemanticTree, SemanticValue};

/// Specifies where the input for speech recognition should come from.
pub enum RecognitionInput {
    /// Listen to the default recording device on the system
    Default,
    /// Read from the given stream
    Stream(AudioStream),
}

impl RecognitionInput {
    fn to_sapi(self) -> Result<IUnknown> {
        Ok(match self {
            Self::Default => {
                Category::new(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Speech\AudioInput")?
                    .default_token()?
                    .to_sapi()
                    .0
            }
            Self::Stream(stream) => stream.to_sapi().0,
        })
    }
}

/// The in-process speech recognition engine.
pub struct Recognizer {
    intf: Intf<ISpRecognizer>,
    pauser: RecognitionPauser,
    global_pause: Mutex<Option<ScopedPause>>,
}

impl Recognizer {
    /// Creates a new recognition engine, configured to listen to the default recording device.
    pub fn new() -> Result<Self> {
        let intf: ISpRecognizer =
            unsafe { CoCreateInstance(&SpInprocRecognizer, None, CLSCTX_ALL) }?;
        unsafe { intf.SetInput(RecognitionInput::Default.to_sapi()?, false) }?;
        Ok(Self {
            pauser: RecognitionPauser::new(intf.clone()),
            intf: Intf(intf),
            global_pause: Mutex::new(None),
        })
    }

    /// Configures the recognizer to listen to the given input.
    pub fn set_input(&self, input: RecognitionInput, allow_fmt_changes: bool) -> Result<()> {
        unsafe { self.intf.SetInput(input.to_sapi()?, allow_fmt_changes) }
    }

    /// Enables or disables recognition.
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let mut global_pause = self.global_pause.lock().unwrap();
        if global_pause.is_none() != enabled {
            if enabled {
                *global_pause = None;
            } else {
                *global_pause = Some(self.pauser.pause()?);
            }
        }
        Ok(())
    }
}

fn reco_state(enabled: bool) -> SPRECOSTATE {
    if enabled {
        SPRST_ACTIVE
    } else {
        SPRST_INACTIVE
    }
}

struct PauserState {
    intf: Intf<ISpRecognizer>,
    pause_count: usize,
}

impl PauserState {
    fn pause(&mut self) -> Result<()> {
        if self.pause_count == 0 {
            unsafe { self.intf.SetRecoState(reco_state(false)) }?;
        }
        self.pause_count += 1;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        if self.pause_count == 1 {
            unsafe { self.intf.SetRecoState(reco_state(true)) }?;
        }
        self.pause_count -= 1;
        Ok(())
    }
}

#[derive(Clone)]
struct RecognitionPauser {
    state: Arc<Mutex<PauserState>>,
}

impl RecognitionPauser {
    fn new(intf: ISpRecognizer) -> Self {
        Self {
            state: Arc::new(Mutex::new(PauserState {
                intf: Intf(intf),
                pause_count: 0,
            })),
        }
    }

    fn pause(&self) -> Result<ScopedPause> {
        ScopedPause::new(self.state.clone())
    }
}

struct ScopedPause {
    state: Arc<Mutex<PauserState>>,
}

impl ScopedPause {
    fn new(state: Arc<Mutex<PauserState>>) -> Result<Self> {
        {
            state.lock().unwrap().pause()?;
        }
        Ok(Self {
            state,
        })
    }
}

impl Drop for ScopedPause {
    fn drop(&mut self) {
        // The following call is expected to succeed, but failure shouldn't cause panic
        let _ = self.state.lock().unwrap().resume();
    }
}
