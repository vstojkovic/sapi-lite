use std::time::Duration;

use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

use crate::tts::{Voice, VoiceSelector};

use super::{Pitch, Rate, SayAs, Speech, Volume};

pub enum SpeechBuilder {
    Text(String),
    Xml(EventWriter<Vec<u8>>),
}

impl SpeechBuilder {
    pub fn new() -> Self {
        Self::Text(String::new())
    }

    pub fn start_emphasis(self) -> Self {
        self.append_xml(XmlEvent::start_element("emph").into())
    }

    pub fn start_pitch<P: Into<Pitch>>(self, pitch: P) -> Self {
        self.append_xml(
            XmlEvent::start_element("pitch").attr("absmiddle", &pitch.into().to_string()).into(),
        )
    }

    pub fn start_rate<R: Into<Rate>>(self, rate: R) -> Self {
        self.append_xml(
            XmlEvent::start_element("rate").attr("absspeed", &rate.into().to_string()).into(),
        )
    }

    pub fn start_voice(self, voice: &Voice) -> Self {
        let mut selector = VoiceSelector::new();
        if let Some(name) = voice.name() {
            selector = selector.name_eq(name.to_string_lossy());
        }
        self.select_and_start_voice(selector, None)
    }

    pub fn select_and_start_voice(
        self,
        required: VoiceSelector,
        optional: Option<VoiceSelector>,
    ) -> Self {
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

    pub fn start_volume<V: Into<Volume>>(self, volume: V) -> Self {
        self.append_xml(
            XmlEvent::start_element("volume").attr("level", &volume.into().to_string()).into(),
        )
    }

    // TODO: What about punctuation, whitespace, etc?
    pub fn say<S: AsRef<str>>(mut self, text: S) -> Self {
        match self {
            Self::Text(ref mut contents) => {
                contents.push_str(text.as_ref());
            }
            Self::Xml(ref mut writer) => {
                writer.write(text.as_ref()).unwrap();
            }
        };
        self
    }

    pub fn say_as<S: AsRef<str>>(self, text: S, ctx: SayAs) -> Self {
        self.append_xml(XmlEvent::start_element("context").attr("id", ctx.sapi_id()).into())
            .say(text)
            .end_element("context")
    }

    pub fn pronounce<S: AsRef<str>>(self, pronunciation: S) -> Self {
        self.append_xml(XmlEvent::start_element("pron").attr("sym", pronunciation.as_ref()).into())
            .end_element("pron")
    }

    pub fn silence(self, duration: Duration) -> Self {
        let millis = duration.as_millis();
        if millis == 0 {
            return self;
        }

        self.append_xml(XmlEvent::start_element("silence").attr("msec", &millis.to_string()).into())
            .end_element("silence")
    }

    pub fn end_emphasis(self) -> Self {
        self.end_element("emph")
    }

    pub fn end_pitch(self) -> Self {
        self.end_element("pitch")
    }

    pub fn end_rate(self) -> Self {
        self.end_element("rate")
    }

    pub fn end_voice(self) -> Self {
        self.end_element("voice")
    }

    pub fn end_volume(self) -> Self {
        self.end_element("volume")
    }

    pub fn build<'s>(self) -> Speech<'s> {
        match self {
            Self::Text(contents) => Speech::Text(contents.into()),
            Self::Xml(writer) => {
                Speech::Xml(String::from_utf8(writer.into_inner()).unwrap().into())
            }
        }
    }

    fn end_element(self, name: &str) -> Self {
        self.append_xml(XmlEvent::end_element().name(name).into())
    }

    fn append_xml(mut self, event: XmlEvent) -> Self {
        match self {
            Self::Text(contents) => {
                let mut writer = EventWriter::new_with_config(
                    Vec::new(),
                    EmitterConfig::new()
                        .keep_element_names_stack(false)
                        .write_document_declaration(false),
                );
                writer.write(contents.as_ref()).unwrap();
                writer.write(event).unwrap();
                Self::Xml(writer)
            }
            Self::Xml(ref mut writer) => {
                writer.write(event).unwrap();
                self
            }
        }
    }
}

impl<'s> From<SpeechBuilder> for Speech<'s> {
    fn from(builder: SpeechBuilder) -> Self {
        builder.build()
    }
}
