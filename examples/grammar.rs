//! This showcases all the different rules you can define in a speech recognition grammar.

use std::collections::HashMap;
use std::time::Duration;

use sapi_lite::stt::{Recognizer, Rule, SyncContext};

fn main() {
    // Initialize SAPI.
    sapi_lite::initialize().unwrap();

    // Create a speech recognizer and recognition context.
    let recog = Recognizer::new().unwrap();
    let ctx = SyncContext::new(&recog).unwrap();

    // Create the grammar and load it into the context.
    let grammar = ctx
        .grammar_builder()
        .add_rule(
            // The root of the semantic tree will be a string that identifies the command user
            // issued.
            &Rule::semantic(
                // For this command, it's "set_color".
                "set_color",
                // This rule has several parts, so we'll use a sequence to represent them.
                &Rule::sequence(vec![
                    // The first part is the word "set".
                    &Rule::text("set"),
                    // The word "the" is optional, so we'll say it can be repeated 0..=1 times.
                    &Rule::repeat(..=1, &Rule::text("the")),
                    // Which color does the user want to change, foreground or background? We'll use
                    // the associated semantic value as a key in a hash map.
                    &Rule::choice(vec![
                        &Rule::semantic("bg", &Rule::text("background")),
                        &Rule::semantic("fg", &Rule::text("foreground")),
                    ]),
                    // The word "color" is also optional.
                    &Rule::repeat(..=1, &Rule::text("color")),
                    // But the word "to" is mandatory.
                    &Rule::text("to"),
                    // We want to offer 8 possible colors, so we'll use a choice to represent that.
                    // Each alternative will be a semantic rule that maps an RGB integer value to
                    // a text rule with the name of that color.
                    &Rule::choice(vec![
                        &Rule::semantic(0x000000, &Rule::text("black")),
                        &Rule::semantic(0x0000ff, &Rule::text("blue")),
                        &Rule::semantic(0x00ff00, &Rule::text("green")),
                        &Rule::semantic(0x00ffff, &Rule::text("cyan")),
                        &Rule::semantic(0xff0000, &Rule::text("red")),
                        &Rule::semantic(0xff00ff, &Rule::text("magenta")),
                        &Rule::semantic(0xffff00, &Rule::text("yellow")),
                        &Rule::semantic(0xffffff, &Rule::text("white")),
                    ]),
                ]),
            ),
        )
        .add_rule(&Rule::semantic("quit", &Rule::text("quit")))
        .build()
        .unwrap();

    // Enable the grammar so its phrases can be recognized.
    grammar.set_enabled(true).unwrap();

    // Prime the hash map with the recognizable keys that we'll extract from the semantic tree.
    let mut colors = HashMap::new();
    colors.insert("bg".to_string(), 0);
    colors.insert("fg".to_string(), 0);

    loop {
        // Print the current RGB values for foreground and background color.
        println!(
            "Background: #{:08x}    Foreground: #{:08x}",
            colors["bg"], colors["fg"]
        );

        // Wait up to 10 seconds for the engine to recognize a command.
        let result = ctx.recognize(Duration::from_secs(10)).unwrap();

        if let Some(phrase) = result {
            // Given the grammar we've loaded, there should be only one top-level semantic tag, and
            // that one holds a string that identifies the command the user issued. Its children
            // will hold the values specific to that command.
            let command = &phrase.semantics[0];

            // Note that `SemanticValue` implements `PartialEq` for comparisons with several other
            // types.
            if command.value == "quit" {
                break;
            }

            // The structure of the semantic information is dictated by the nesting of the grammar
            // rules. For example, the phrase "set the background to red" should produce "bg" as the
            // first child of the top-level semantic value "set_color", and 0xff0000 as its second
            // child, because those semantic rules are nested inside choice rules, which are nested
            // inside the sequence rule which is inside the semantic rule that maps "set_color" to
            // this phrase.
            if command.value == "set_color" {
                // Extract the hash map key from the semantic tree.
                let key = command[0].value.as_string().unwrap().to_string_lossy();

                // Extract the RGB value from the semantic tree.
                let value = command[1].value.as_int().unwrap();

                // Set the value in the hash map.
                *colors.get_mut(key.as_ref()).unwrap() = *value;
            }
        }
    }

    // We don't need SAPI anymore. Clean up and free the resources.
    sapi_lite::finalize();
}
