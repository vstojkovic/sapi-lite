use std::sync::{Arc, Mutex};

use windows as Windows;
use Windows::Win32::Media::Speech::{
    ISpRecognizer, SpInprocRecognizer, SPRECOSTATE, SPRST_ACTIVE, SPRST_INACTIVE,
};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::token::Category;
use crate::Result;

mod context;
mod grammar;
mod phrase;
mod semantics;

pub use context::{Context, EventHandler, EventfulContext, SyncContext};
pub use grammar::{Grammar, GrammarBuilder, Rule};
pub use phrase::Phrase;
pub use semantics::{SemanticString, SemanticTree, SemanticValue};

pub struct Recognizer {
    intf: ISpRecognizer,
    pauser: RecognitionPauser,
    global_pause: Mutex<Option<ScopedPause>>,
}

impl Recognizer {
    pub fn new() -> Result<Self> {
        let intf: ISpRecognizer =
            unsafe { CoCreateInstance(&SpInprocRecognizer, None, CLSCTX_ALL) }?;

        let category = Category::new(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Speech\AudioInput")?;
        let token = category.default_token()?;
        unsafe { intf.SetInput(token.intf, false) }?;
        Ok(Self {
            pauser: RecognitionPauser::new(intf.clone()),
            intf,
            global_pause: Mutex::new(None),
        })
    }

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
    intf: ISpRecognizer,
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
                intf,
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
