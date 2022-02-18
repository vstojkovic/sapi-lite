use std::ops::Deref;

use tokio::sync::mpsc::Receiver;

use crate::stt::{Context, EventfulContext, Phrase, Recognizer};
use crate::Result;

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-stt")))]
/// A recognition context that can be awaited.
pub struct AsyncContext {
    base: EventfulContext,
    rx: Receiver<Phrase>,
}

impl AsyncContext {
    /// Creates a new recognition context for the given recognizer, configured to buffer up to the
    /// given number of recognized phrases. If a new phrase is recognized while the buffer is full,
    /// it will be silently dropped.
    pub fn new(recognizer: &Recognizer, buffer: usize) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel::<Phrase>(buffer);
        let handler = move |phrase| {
            let _ = tx.try_send(phrase);
        };
        Ok(Self {
            base: EventfulContext::new(recognizer, handler)?,
            rx,
        })
    }

    /// Completes when the engine recognizes a phrase.
    pub async fn recognize(&mut self) -> Phrase {
        self.rx.recv().await.unwrap()
    }
}

impl Deref for AsyncContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
