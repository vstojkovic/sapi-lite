//! This example shows how to recognize speech in an audio file instead of listening to the default
//! recording device.
//!
//! Note that an easy alternative to recording your own voice to an audio file is to run the
//! `stream_tts` example and then use the file it produced.

use std::env;
use std::time::Duration;

use sapi_lite::audio::{AudioFormat, AudioStream, BitRate, Channels, SampleRate};
use sapi_lite::stt::{Context, Grammar, RecognitionInput, Recognizer, Rule, SyncContext};

// Create a grammar with the phrases for the engine to recognize.
fn create_grammar(ctx: &Context) -> Grammar {
    ctx.grammar_builder()
        .add_rule(&Rule::sequence(vec![
            &Rule::text("Harry Potter and the"),
            &Rule::choice(vec![
                &Rule::semantic(1, &Rule::text("Philosopher's Stone")),
                &Rule::semantic(2, &Rule::text("Chamber of Secrets")),
                &Rule::semantic(3, &Rule::text("Prisoner of Azkaban")),
                &Rule::semantic(4, &Rule::text("Goblet of Fire")),
                &Rule::semantic(5, &Rule::text("Order of the Phoenix")),
                &Rule::semantic(6, &Rule::text("Half-Blood Prince")),
                &Rule::semantic(7, &Rule::text("Deathly Hallows")),
            ]),
        ]))
        .build()
        .unwrap()
}

fn main() {
    // Grab the name of the input file from the command-line arguments.
    let path = {
        if let Some(arg) = env::args().nth(1) {
            arg
        } else {
            println!("Please specify the input pathname on the command line.");
            return;
        }
    };

    // Initialize SAPI.
    sapi_lite::initialize().unwrap();

    // Define the input audio format.
    let audio_fmt = AudioFormat {
        sample_rate: SampleRate::Hz44100,
        bit_rate: BitRate::Bits8,
        channels: Channels::Stereo,
    };

    // Open the file and create the audio stream that will read from it.
    let stream = AudioStream::open_file(path, &audio_fmt).unwrap();

    // Create a speech recognizer.
    let recog = Recognizer::new().unwrap();

    // Configure the recognizer to read from the file-backed audio stream.
    recog
        .set_input(RecognitionInput::Stream(stream), false)
        .unwrap();

    // Create the recognition context, load a grammar into it, and enable the grammar so the engine
    // will recognize its phrases.
    let ctx = SyncContext::new(&recog).unwrap();
    let grammar = create_grammar(&ctx);
    grammar.set_enabled(true).unwrap();

    // Try to recognize one of the configured phrases. If the recognition was successful, print out
    // the recognized phrase and associated semantic information.
    if let Some(phrase) = ctx.recognize(Duration::from_secs(5)).unwrap() {
        println!(
            "\"{}\" is the book #{} in the series.",
            phrase.text.to_string_lossy(),
            phrase.semantics[0].value.as_int().unwrap()
        );
    } else {
        println!("I don't recognize that Harry Potter book.");
    }

    // We don't need SAPI anymore. Clean up and free the resources.
    sapi_lite::finalize();
}
