//! A bare-bones TTS example.

use sapi_lite::tts::SyncSynthesizer;

fn main() {
    // Initialize SAPI.
    sapi_lite::initialize().unwrap();

    // Create a speech synthesizer.
    let synth = SyncSynthesizer::new().unwrap();

    // Speak the phrase and wait until the speech is finished.
    synth.speak("Hello, world!", None).unwrap();

    // We don't need SAPI anymore. Clean up and free the resources.
    sapi_lite::finalize();
}
