use std::borrow::{Borrow, Cow};
use std::time::Duration;

use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};
use windows as Windows;
use Windows::Win32::Media::Speech::{SPF_DEFAULT, SPF_IS_XML, SPF_PARSE_SAPI};

use super::voice::{VoiceSelector, Voice};

pub enum Speech<'s> {
    Text(Cow<'s, str>),
    Xml(Cow<'s, str>),
}

impl<'s> Speech<'s> {
    pub(crate) fn flags(&self) -> u32 {
        (match self {
            Self::Text(_) => SPF_DEFAULT.0,
            Self::Xml(_) => (SPF_IS_XML.0 | SPF_PARSE_SAPI.0),
        }) as u32
    }

    pub(crate) fn contents(&self) -> &str {
        match self {
            Self::Text(cow) => cow.borrow(),
            Self::Xml(cow) => cow.borrow(),
        }
    }
}

impl<'s> From<&'s str> for Speech<'s> {
    fn from(s: &'s str) -> Self {
        Self::Text(s.into())
    }
}

impl<'s> From<String> for Speech<'s> {
    fn from(s: String) -> Self {
        Self::Text(s.into())
    }
}

pub enum SayAs<'s> {
    DateMDY,
    DateDMY,
    DateYMD,
    DateYM,
    DateMY,
    DateDM,
    DateMD,
    DateYear,
    Time,
    NumberCardinal,
    NumberDigit,
    NumberFraction,
    NumberDecimal,
    PhoneNumber,
    Custom(&'s str),
}

impl<'s> SayAs<'s> {
    fn sapi_id(&self) -> &str {
        match self {
            Self::DateMDY => "date_mdy",
            Self::DateDMY => "date_dmy",
            Self::DateYMD => "date_ymd",
            Self::DateYM => "date_ym",
            Self::DateMY => "date_my",
            Self::DateDM => "date_dm",
            Self::DateMD => "date_md",
            Self::DateYear => "date_year",
            Self::Time => "time",
            Self::NumberCardinal => "number_cardinal",
            Self::NumberDigit => "number_digit",
            Self::NumberFraction => "number_fraction",
            Self::NumberDecimal => "number_decimal",
            Self::PhoneNumber => "phone_number",
            Self::Custom(s) => s,
        }
    }
}

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

    pub fn start_pitch(self, pitch: i32) -> Self {
        self.append_xml(
            XmlEvent::start_element("pitch")
                .attr("absmiddle", &pitch.clamp(-10, 10).to_string())
                .into(),
        )
    }

    pub fn start_rate(self, rate: i32) -> Self {
        self.append_xml(
            XmlEvent::start_element("rate")
                .attr("absspeed", &rate.clamp(-10, 10).to_string())
                .into(),
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

    pub fn start_volume(self, volume: i32) -> Self {
        self.append_xml(
            XmlEvent::start_element("volume")
                .attr("level", &volume.clamp(0, 100).to_string())
                .into(),
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
        match builder {
            SpeechBuilder::Text(contents) => Self::Text(contents.into()),
            SpeechBuilder::Xml(writer) => {
                Self::Xml(String::from_utf8(writer.into_inner()).unwrap().into())
            }
        }
    }
}
