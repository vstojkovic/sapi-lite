use std::ops::Deref;

use tokio::sync::mpsc::Receiver;

use crate::Result;

use super::{EventfulContext, Recognizer, Phrase};
use super::context::Context;

pub struct AsyncContext {
    base: EventfulContext,
    rx: Receiver<Phrase>,
}

impl AsyncContext {
    pub fn new(recognizer: &Recognizer, buffer: usize) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel::<Phrase>(buffer);
        let handler = move |phrase| {
            let _ = tx.try_send(phrase);
        };
        Ok(Self {
            base: EventfulContext::new(recognizer, handler)?,
            rx
        })
    }

    pub async fn recognize(&mut self) -> Option<Phrase> {
        self.rx.recv().await
    }
}

impl Deref for AsyncContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
