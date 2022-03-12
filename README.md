# `sapi-lite`

**A simplified wrapper around Microsoft's Speech API (SAPI) library.**

The goal of this crate is to expose a subset of SAPI features in a way that is easy to use in Rust.
It does not aim to provide the full set of features SAPI offers. Since the focus is on relative ease
of use (compared to dealing with COM directly) and simplicity of API, many SAPI features are missing
in this crate.

## Example

```rust
use sapi_lite::stt::{Recognizer, Rule, SyncContext};
use sapi_lite::tts::{SyncSynthesizer};
use std::time::Duration;

sapi_lite::initialize().unwrap();

let synth = SyncSynthesizer::new().unwrap();
synth
    .speak("The Doors of Durin, Lord of Moria. Speak, friend, and enter.", None)
    .unwrap();

let recog = Recognizer::new().unwrap();
let ctx = SyncContext::new(&recog).unwrap();

let grammar = ctx
    .grammar_builder()
    .add_rule(&Rule::text("friend"))
    .build()
    .unwrap();
grammar.set_enabled(true).unwrap();

if let Some(phrase) = ctx.recognize(Duration::from_secs(5)).unwrap() {
    println!(
        "The gate swings open. Welcome to Moria, {}.",
        phrase.text.to_string_lossy()
    );
} else {
    println!("The gate to Moria remains shut.")
}

sapi_lite::finalize();
```
