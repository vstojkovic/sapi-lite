use std::ffi::OsString;
use std::time::{Duration, Instant};

use sapi_lite::audio::{AudioFormat, AudioStream, BitRate, Channels, MemoryStream, SampleRate};
use sapi_lite::stt::{
    Context, Grammar, Phrase, RecognitionInput, Recognizer, Rule, SemanticTree, SemanticValue,
    SyncContext,
};
use sapi_lite::tts::{Speech, SpeechOutput, SyncSynthesizer};

#[test]
fn test_round_trip() {
    sapi_lite::initialize().unwrap();

    let audio_fmt = AudioFormat {
        sample_rate: SampleRate::Hz8000,
        bit_rate: BitRate::Bits8,
        channels: Channels::Mono,
    };
    let stream = MemoryStream::new(None).unwrap();
    let speech = "have a very very good evening";

    let timeout = speak(
        speech,
        create_output(stream.try_clone().unwrap(), &audio_fmt),
    );
    let phrase = recognize(create_input(stream, &audio_fmt), timeout).unwrap();

    assert_eq!(speech, phrase.text);
    assert_eq!(
        vec![tree("how_good", vec![leaf(1), leaf(1)]), leaf("pm")],
        phrase.semantics
    );

    sapi_lite::finalize();
}

fn create_output(stream: MemoryStream, audio_fmt: &AudioFormat) -> SpeechOutput {
    SpeechOutput::Stream(AudioStream::from_stream(stream, &audio_fmt).unwrap())
}

fn create_input(stream: MemoryStream, audio_fmt: &AudioFormat) -> RecognitionInput {
    RecognitionInput::Stream(AudioStream::from_stream(stream, &audio_fmt).unwrap())
}

fn speak<'s, S: Into<Speech<'s>>>(speech: S, output: SpeechOutput) -> Duration {
    let start = Instant::now();
    let synth = SyncSynthesizer::new().unwrap();
    synth.set_output(output, false).unwrap();
    synth.speak(speech, None).unwrap();
    std::cmp::min(start.elapsed() * 2, Duration::from_millis(100))
}

fn recognize(input: RecognitionInput, timeout: Duration) -> Option<Phrase> {
    let recog = Recognizer::new().unwrap();
    recog.set_input(input, false).unwrap();
    let ctx = SyncContext::new(&recog).unwrap();
    let grammar = create_grammar(&ctx);
    grammar.set_enabled(true).unwrap();
    ctx.recognize(timeout).unwrap()
}

fn create_grammar(ctx: &Context) -> Grammar {
    ctx.grammar_builder()
        .add_rule(&Rule::sequence(
            &[
                &Rule::text("have a"),
                &Rule::semantic(
                    "how_good",
                    &Rule::repeat(0..=3, &Rule::semantic(1, &Rule::text("very"))),
                ),
                &Rule::text("good".to_string()),
                &Rule::choice(
                    &[
                        &Rule::semantic("am", &Rule::text("morning")),
                        &Rule::semantic("pm", &Rule::text("evening")),
                    ][..],
                ),
            ][..],
        ))
        .build()
        .unwrap()
}

fn tree<V: Into<SemanticValue<OsString>>>(value: V, children: Vec<SemanticTree>) -> SemanticTree {
    SemanticTree {
        value: value.into(),
        children,
    }
}

fn leaf<V: Into<SemanticValue<OsString>>>(value: V) -> SemanticTree {
    tree(value, vec![])
}
