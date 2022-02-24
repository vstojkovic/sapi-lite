use std::fmt;
use std::time::Duration;

use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

use crate::tts::{Voice, VoiceSelector};

use super::{Pitch, Rate, SayAs, Speech, Volume};

/// Helper type that can construct a [`Speech`] from a sequence of rendering instructions.
///
/// NOTE: Although any complex speech is encoded as XML, the builder performs no validation. This is
/// because SAPI itself is very lax when processing speech. For example, SAPI will be perfectly
/// happy to render the following XML:
/// ```xml
/// <emph><volume level="50">Hello</emph>world</volume>
/// ```
pub struct SpeechBuilder {
    state: SpeechBuilderState,
}

enum SpeechBuilderState {
    Text(String),
    Xml(EventWriter<Vec<u8>>),
}

impl SpeechBuilder {
    /// Constructs a new, empty instance.
    pub fn new() -> Self {
        Self {
            state: SpeechBuilderState::Text(String::new()),
        }
    }

    /// Emphasizes all subsequent speech until the corresponding
    /// [`end_emphasis`](SpeechBuilder::end_emphasis) call.
    pub fn start_emphasis(&mut self) -> &mut Self {
        self.append_xml(XmlEvent::start_element("emph").into())
    }

    /// Changes the pitch of all subsequent speech until the corresponding
    /// [`end_pitch`](SpeechBuilder::end_pitch) call.
    pub fn start_pitch<P: Into<Pitch>>(&mut self, pitch: P) -> &mut Self {
        self.append_xml(
            XmlEvent::start_element("pitch").attr("absmiddle", &pitch.into().to_string()).into(),
        )
    }

    /// Changes the rate of all subsequent speech until the corresponding
    /// [`end_rate`](SpeechBuilder::end_rate) call.
    pub fn start_rate<R: Into<Rate>>(&mut self, rate: R) -> &mut Self {
        self.append_xml(
            XmlEvent::start_element("rate").attr("absspeed", &rate.into().to_string()).into(),
        )
    }

    /// Switches to the specified voice until the corresponding
    /// [`end_voice`](SpeechBuilder::end_voice) call.
    pub fn start_voice(&mut self, voice: &Voice) -> &mut Self {
        let mut selector = VoiceSelector::new();
        if let Some(name) = voice.name() {
            selector = selector.name_eq(name.to_string_lossy());
        }
        self.select_and_start_voice(selector, None)
    }

    /// Switches to a voice that matches the specified criteria until the corresponding
    /// [`end_voice`](SpeechBuilder::end_voice) call. For the explanation of `required` and
    /// `optional` criteria, see [`installed_voices`](crate::tts::installed_voices).
    pub fn select_and_start_voice(
        &mut self,
        required: VoiceSelector,
        optional: Option<VoiceSelector>,
    ) -> &mut Self {
        let mut event = XmlEvent::start_element("voice");

        let required_expr = required.into_sapi_expr();
        if !required_expr.is_empty() {
            event = event.attr("required", &required_expr);
        }

        let optional_expr = optional.map(VoiceSelector::into_sapi_expr);
        if let Some(optional_expr) = optional_expr.as_ref() {
            if !optional_expr.is_empty() {
                event = event.attr("optional", optional_expr);
            }
        }

        self.append_xml(event.into())
    }

    /// Changes the volume of all subsequent speech until the corresponding
    /// [`end_rate`](SpeechBuilder::end_rate) call.
    pub fn start_volume<V: Into<Volume>>(&mut self, volume: V) -> &mut Self {
        self.append_xml(
            XmlEvent::start_element("volume").attr("level", &volume.into().to_string()).into(),
        )
    }

    /// Appends text to pronounce.
    pub fn say<S: AsRef<str>>(&mut self, text: S) -> &mut Self {
        // TODO: What about punctuation, whitespace, etc?
        match &mut self.state {
            SpeechBuilderState::Text(contents) => {
                contents.push_str(text.as_ref());
            }
            SpeechBuilderState::Xml(writer) => {
                writer.write(text.as_ref()).unwrap();
            }
        };
        self
    }

    /// Appends text to pronounce, along witha hint on how to pronounce it.
    pub fn say_as<S: AsRef<str>>(&mut self, text: S, ctx: SayAs) -> &mut Self {
        self.append_xml(XmlEvent::start_element("context").attr("id", ctx.sapi_id()).into())
            .say(text)
            .end_element("context")
    }

    /// Appends a specific pronunciation to render. The pronunciation specification depends on the
    /// language of the current voice. For example, "m ah dh ax r" in American English is pronounced
    /// as "mother".
    pub fn pronounce<S: AsRef<str>>(&mut self, pronunciation: S) -> &mut Self {
        self.append_xml(XmlEvent::start_element("pron").attr("sym", pronunciation.as_ref()).into())
            .end_element("pron")
    }

    /// Appends a silence with a specified duration. Does not support sub-millisecond precision.
    pub fn silence(&mut self, duration: Duration) -> &mut Self {
        let millis = duration.as_millis();
        if millis == 0 {
            return self;
        }

        self.append_xml(XmlEvent::start_element("silence").attr("msec", &millis.to_string()).into())
            .end_element("silence")
    }

    /// Ends the effect of the corresponding [`start_emphasis`](SpeechBuilder::start_emphasis) call.
    pub fn end_emphasis(&mut self) -> &mut Self {
        self.end_element("emph")
    }

    /// Ends the effect of the corresponding [`start_pitch`](SpeechBuilder::start_pitch) call.
    pub fn end_pitch(&mut self) -> &mut Self {
        self.end_element("pitch")
    }

    /// Ends the effect of the corresponding [`start_rate`](SpeechBuilder::start_rate) call.
    pub fn end_rate(&mut self) -> &mut Self {
        self.end_element("rate")
    }

    /// Ends the effect of the corresponding [`start_voice`](SpeechBuilder::start_voice) or
    /// [`select_and_start_voice`](SpeechBuilder::select_and_start_voice) call.
    pub fn end_voice(&mut self) -> &mut Self {
        self.end_element("voice")
    }

    /// Ends the effect of the corresponding [`start_volume`](SpeechBuilder::start_volume) call.
    pub fn end_volume(&mut self) -> &mut Self {
        self.end_element("volume")
    }

    /// Builds the [`Speech`] from instructions received so far. Clears the contents of the builder.
    pub fn build<'s>(&mut self) -> Speech<'s> {
        match std::mem::replace(&mut self.state, SpeechBuilderState::Text(String::new())) {
            SpeechBuilderState::Text(contents) => Speech::Text(contents.into()),
            SpeechBuilderState::Xml(writer) => {
                Speech::Xml(String::from_utf8(writer.into_inner()).unwrap().into())
            }
        }
    }

    fn end_element(&mut self, name: &str) -> &mut Self {
        self.append_xml(XmlEvent::end_element().name(name).into())
    }

    fn append_xml(&mut self, event: XmlEvent) -> &mut Self {
        match &mut self.state {
            SpeechBuilderState::Text(contents) => {
                let mut writer = EventWriter::new_with_config(
                    Vec::new(),
                    EmitterConfig::new()
                        .keep_element_names_stack(false)
                        .write_document_declaration(false),
                );
                writer.write(contents.as_ref()).unwrap();
                writer.write(event).unwrap();
                self.state = SpeechBuilderState::Xml(writer);
            }
            SpeechBuilderState::Xml(writer) => {
                writer.write(event).unwrap();
            }
        }
        self
    }
}

impl fmt::Write for SpeechBuilder {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.say(s);
        Ok(())
    }
}

impl<'s> From<SpeechBuilder> for Speech<'s> {
    fn from(mut builder: SpeechBuilder) -> Self {
        builder.build()
    }
}

impl<'s> From<&mut SpeechBuilder> for Speech<'s> {
    fn from(builder: &mut SpeechBuilder) -> Self {
        builder.build()
    }
}
