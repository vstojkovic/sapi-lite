use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use tokio::sync::oneshot::{channel, Sender};

use crate::tts::{EventfulSynthesizer, Speech, Synthesizer};
use crate::Result;

enum PendingSpeech {
    Waiting(Sender<()>),
    Finished,
}

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-tts")))]
/// A speech synthesizer that returns a future for every speech it renders.
pub struct AsyncSynthesizer {
    base: EventfulSynthesizer,
    pending_speeches: Arc<Mutex<HashMap<u32, PendingSpeech>>>,
}

impl AsyncSynthesizer {
    /// Creates a new synthesizer, configured to output its speech to the default audio device.
    pub fn new() -> Result<Self> {
        let pending_speeches = Arc::new(Mutex::new(HashMap::<u32, PendingSpeech>::new()));
        let handler = {
            let pending_speeches = pending_speeches.clone();
            move |id| {
                let mut map = pending_speeches.lock().unwrap();
                if let Some(PendingSpeech::Waiting(tx)) = map.remove(&id) {
                    let _ = tx.send(());
                } else {
                    map.insert(id, PendingSpeech::Finished);
                }
            }
        };
        Ok(Self {
            base: EventfulSynthesizer::new(handler)?,
            pending_speeches,
        })
    }

    /// Completes when the synthesizer finished rendering the given speech.
    pub async fn speak<'s, S: Into<Speech<'s>>>(&self, speech: S) -> Result<()> {
        let id = self.base.speak(speech)?;
        let rx = {
            let mut map = self.pending_speeches.lock().unwrap();
            if let Some(PendingSpeech::Finished) = map.remove(&id) {
                return Ok(());
            }
            let (tx, rx) = channel();
            map.insert(id, PendingSpeech::Waiting(tx));
            rx
        };
        let _ = rx.await;
        Ok(())
    }
}

impl Deref for AsyncSynthesizer {
    type Target = Synthesizer;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
