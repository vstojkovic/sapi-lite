//! This example showcases how to use event-based speech recognition. The speech recognition engine
//! is configured to recognize the word "half". Every time the word is recognized, the engine will
//! call the given callback, which will then increment a counter. When the user signals they are
//! finished reading, the program will print out the value of the counter.

use std::io::{stdin, BufRead};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sapi_lite::stt::{EventfulContext, Recognizer, Rule};

const INSTRUCTIONS: &'static str = r#"
Choose a text and read it out loud. For example:

"I don't know half of you half as well as I should like;
and I like less than half of you half as well as you deserve."

When you're done, press ENTER to see how many times you said the word "half".
"#;

fn main() {
    // Initialize SAPI.
    sapi_lite::initialize().unwrap();

    // Print out the instructions for the user.
    println!("{}", INSTRUCTIONS);

    // Create the counter to use in our event callback.
    let counter = Arc::new(AtomicUsize::new(0));

    // Count how many times the user said "half" until the user presses ENTER.
    count_halves(&counter);

    // Display the value of the counter.
    let count = counter.load(Ordering::Relaxed);
    println!(
        "You said the word \"half\" exactly {} {}.",
        count,
        if count == 1 { "time" } else { "times" }
    );

    // We don't need SAPI anymore. Clean up and free the resources.
    sapi_lite::finalize();
}

fn count_halves(counter: &Arc<AtomicUsize>) {
    // Note that the handler needs to have a static lifetime. Also note that the handler discards
    // the recognized phrase it receives as its argument, since we have only one possible phrase and
    // we don't need to extract any data from it.
    let handler = {
        let counter = counter.clone();
        move |_| {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    };

    // Create a speech recognizer.
    let recog = Recognizer::new().unwrap();

    // Create a recognition context with the given handler.
    let ctx = EventfulContext::new(&recog, handler).unwrap();

    // Load the grammar into the context.
    let grammar = ctx
        .grammar_builder()
        .add_rule(&Rule::text("half"))
        .build()
        .unwrap();

    // Enable the grammar so its phrase can be recognized.
    grammar.set_enabled(true).unwrap();

    // Block the main thread while waiting for the user to signal they're done reading. Note that
    // the handler will be called from a SAPI thread.
    stdin().lock().lines().next();

    // Note that the recognizer, the context, and the grammar will be dropped here. It's important
    // to ensure all the SAPI resources are dropped before calling `sapi_lite::finalize()`.
}
