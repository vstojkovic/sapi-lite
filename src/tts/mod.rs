mod speech;
mod synthesizer;
mod voice;

pub use self::speech::{Pitch, Rate, SayAs, Speech, SpeechBuilder, Volume};
pub use self::synthesizer::{
    EventHandler, EventfulSynthesizer, SpeechOutput, SyncSynthesizer, Synthesizer,
};
pub use self::voice::{installed_voices, Voice, VoiceAge, VoiceGender, VoiceSelector};
