//! This examples shows how to iterate over the available TTS voices.

use std::io::{self, Write};

use sapi_lite::tts::{installed_voices, SyncSynthesizer, Voice};

fn choose_voice() -> Option<Voice> {
    // Collect all the install voices, without any filters or sorting preferences.
    let mut voices: Vec<Voice> = installed_voices(None, None).unwrap().collect();

    // Display the list to the user.
    println!("Available voices:");
    for (idx, voice) in voices.iter().enumerate() {
        println!("{}) {}", idx + 1, voice.name().unwrap().to_string_lossy());
    }

    // Prompt the user to select a voice from the list.
    print!("Choose a voice: ");
    io::stdout().flush().unwrap();

    // Get the user's selection.
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();

    // If the selection is valid, return the selected voice.
    let selected_idx = line.trim_end().parse::<usize>().ok()?;
    if (selected_idx > 0) && (selected_idx <= voices.len()) {
        Some(voices.swap_remove(selected_idx - 1))
    } else {
        None
    }
}

fn main() {
    // Initialize SAPI.
    sapi_lite::initialize().unwrap();

    // Have the user choose a voice from the list of available voices.
    let voice = match choose_voice() {
        Some(voice) => voice,
        None => return,
    };

    // Create a speech synthesizer.
    let synth = SyncSynthesizer::new().unwrap();

    // Configure the synthesizer to use the selected voice.
    synth.set_voice(&voice).unwrap();

    // Speak a phrase in the selected voice.
    synth
        .speak(
            "The pellet with the poison's in the flagon with the dragon.",
            None,
        )
        .unwrap();

    // We don't need SAPI anymore. Clean up and free the resources.
    sapi_lite::finalize();
}
