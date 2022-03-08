//! An example that shows how to use `SpeechBuilder` to create a speech that can be rendered
//! multiple times. It also showcases some of the commands that control the speech rendering, as
//! well as their interactions with the current configuration of the speech synthesizer.

use std::time::Duration;

use sapi_lite::tts::{SpeechBuilder, SyncSynthesizer, VoiceGender, VoiceSelector};

fn main() {
    // Initialize SAPI.
    sapi_lite::initialize().unwrap();

    // Build a speech with a variety of commands.
    let speech = SpeechBuilder::new()
        .select_and_start_voice(VoiceSelector::new().gender_eq(VoiceGender::Female), None)
        .start_rate(4)
        .say("The pellet with the poison's in the vessel with the pestle.")
        .select_and_start_voice(VoiceSelector::new().gender_eq(VoiceGender::Male), None)
        .say("And the chalice from the palace has the brew that is true.")
        .end_voice()
        .end_rate()
        .start_volume(50)
        .silence(Duration::from_millis(500))
        .say("Just remember that!")
        .end_volume()
        .end_voice()
        .build();

    // Create a speech synthesizer.
    let synth = SyncSynthesizer::new().unwrap();

    // Configure the speech synthesizer to speak more slowly.
    synth.set_rate(-2).unwrap();

    // Render the speech. Note that if there is no voice that satisfies the required criteria given
    // to `select_and_start_voice`, the call to `speak` will fail with a SAPI error. If you want to
    // the synthesizer to try to select a voice that satisfies your criteria without failing if
    // there's no match, specify the criteria as optional instead of required.
    synth.speak(&speech, None).unwrap();

    // Configure the speech synthesizer to speak at a normal rate.
    synth.set_rate(0).unwrap();

    // Configure the speech synthesizer to speak at 80% of its normal volume.
    synth.set_volume(80).unwrap();

    // Render the speech again. Notice that the whole speech is faster and quieter than the previous
    // rendition. This is because the commands given to the speech builder do not override the
    // synthesizer's configuration, but adjust it. For example, `start_volume(50)` will set the
    // speech volume to half of the configured volume level. In this particular call, it will be
    // set to 50% of 80%, i.e. 40% of the maximum speech volume.
    synth.speak(&speech, None).unwrap();

    // We don't need SAPI anymore. Clean up and free the resources.
    sapi_lite::finalize();
}
