use std::ops::Deref;

use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};

use crate::stt::{Context, EventfulContext, Phrase, Recognizer};
use crate::Result;

/// A subscriber that can be awaited for recognized phrases.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-stt")))]
pub struct UnicastSubscriber {
    rx: mpsc::Receiver<Phrase>,
}

impl UnicastSubscriber {
    /// Completes when the engine recognizes a phrase.
    pub async fn recognize(&mut self) -> Phrase {
        self.rx.recv().await.unwrap()
    }
}

/// The result of awaiting a [`BroadcastSubscriber`].
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-stt")))]
#[derive(Debug, Clone, PartialEq)]
pub enum BroadcastResult {
    Phrase(Phrase),
    Lagged(u64),
}

/// A subscriber that can be awaited for recognized phrases.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-stt")))]
pub struct BroadcastSubscriber {
    rx: broadcast::Receiver<Phrase>,
}

impl BroadcastSubscriber {
    /// Completes when the engine recognizes a phrase.
    pub async fn recognize(&mut self) -> BroadcastResult {
        match self.rx.recv().await {
            Ok(phrase) => BroadcastResult::Phrase(phrase),
            Err(RecvError::Lagged(skipped)) => BroadcastResult::Lagged(skipped),
            Err(err) => panic!("{}", err),
        }
    }
}

/// A recognition context paired with a single subscriber that can be awaited for recognition.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-stt")))]
pub struct UnicastContext {
    base: EventfulContext,
}

impl UnicastContext {
    /// Creates a new recognition context for the given recognizer, configured to buffer up to the
    /// given number of recognized phrases. If a new phrase is recognized while the buffer is full,
    /// it will be silently dropped.
    pub fn new(recognizer: &Recognizer, buffer: usize) -> Result<(Self, UnicastSubscriber)> {
        let (tx, rx) = mpsc::channel::<Phrase>(buffer);
        let handler = move |phrase| {
            let _ = tx.try_send(phrase);
        };
        Ok((
            Self {
                base: EventfulContext::new(recognizer, handler)?,
            },
            UnicastSubscriber {
                rx,
            }
        ))
    }
}

impl Deref for UnicastContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// A recognition context paired with a one or more subscribers that can be awaited for recognition.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-stt")))]
pub struct BroadcastContext {
    base: EventfulContext,
    tx: broadcast::Sender<Phrase>,
}

impl BroadcastContext {
    /// Creates a new recognition context for the given recognizer, configured to buffer up to the
    /// given number of recognized phrases. If a new phrase is recognized while one or more
    /// subscribers haven't received it, it will be dropped and those subscribers will yield a
    /// [`BroadcastResult::Lagged`] on next await.
    pub fn new(recognizer: &Recognizer, buffer: usize) -> Result<(Self, BroadcastSubscriber)> {
        let (tx, rx) = broadcast::channel::<Phrase>(buffer);
        let handler = {
            let tx = tx.clone();
            move |phrase| {
               let _ = tx.send(phrase);
            }
        };
        Ok((
            Self {
                base: EventfulContext::new(recognizer, handler)?,
                tx,
            },
            BroadcastSubscriber {
                rx,
            },
        ))
    }

    /// Creates a subscriber for this context.
    pub fn subscribe(&self) -> BroadcastSubscriber {
        BroadcastSubscriber {
            rx: self.tx.subscribe(),
        }
    }
}

impl Deref for BroadcastContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
