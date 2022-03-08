//! A bare-bones speech recognition example.

use std::time::Duration;

use sapi_lite::stt::{Recognizer, Rule, SyncContext};

fn main() {
    // Initialize SAPI.
    sapi_lite::initialize().unwrap();

    println!("The Doors of Durin, Lord of Moria. Speak, friend, and enter.");

    // Create a speech recognizer and a recognition context.
    let recog = Recognizer::new().unwrap();
    let ctx = SyncContext::new(&recog).unwrap();

    // Load a grammar into the recognition context.
    let grammar = ctx
        .grammar_builder()
        .add_rule(&Rule::text("friend"))
        .build()
        .unwrap();

    // Enable the grammar so its phrase can be recognized.
    grammar.set_enabled(true).unwrap();

    // Wait up to 10 seconds for speech recognition.
    if let Some(phrase) = ctx.recognize(Duration::from_secs(5)).unwrap() {
        println!(
            "The gate swings open. Welcome to Moria, {}.",
            phrase.text.to_string_lossy()
        );
    } else {
        println!("The gate to Moria remains shut.")
    }

    // We don't need SAPI anymore. Clean up and free the resources.
    sapi_lite::finalize();
}
