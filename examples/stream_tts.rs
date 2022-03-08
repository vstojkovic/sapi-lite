//! This example shows how to synthesize speech and write the audio to a file instead of playing it
//! on the default audio device.

use std::env;
use std::io::{self, Write};

use sapi_lite::audio::{AudioFormat, AudioStream, BitRate, Channels, SampleRate};
use sapi_lite::tts::{SpeechOutput, SyncSynthesizer};

fn main() {
    // Grab the name of the output file from the command-line arguments.
    let path = {
        if let Some(arg) = env::args().nth(1) {
            arg
        } else {
            println!("Please specify the output pathname on the command line.");
            return;
        }
    };

    // Initialize SAPI.
    sapi_lite::initialize().unwrap();

    // Read the text of the phrase to speak from the standard input.
    print!("Enter the phrase to speak: ");
    io::stdout().flush().unwrap();
    let mut speech = String::new();
    io::stdin().read_line(&mut speech).unwrap();

    // Define the output audio format.
    let audio_fmt = AudioFormat {
        sample_rate: SampleRate::Hz44100,
        bit_rate: BitRate::Bits8,
        channels: Channels::Stereo,
    };

    // Create the output file and the audio stream that will write to it.
    let stream = AudioStream::create_file(path, &audio_fmt).unwrap();

    // Create the speech synthesizer.
    let synth = SyncSynthesizer::new().unwrap();

    // Configure the synthesizer to render the speech to the file-backed audio stream.
    synth
        .set_output(SpeechOutput::Stream(stream), false)
        .unwrap();

    // Render the speech.
    synth.speak(speech, None).unwrap();

    // We don't need SAPI anymore. Clean up and free the resources.
    sapi_lite::finalize();
}
