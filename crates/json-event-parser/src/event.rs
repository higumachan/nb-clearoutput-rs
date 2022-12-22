use std::borrow::Cow;

/// Possible events during JSON parsing
#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum JsonEvent<'a> {
    String(Cow<'a, str>),
    Number(Cow<'a, str>),
    Boolean(bool),
    Null,
    StartArray,
    NextArrayValue,
    EndArray,
    StartObject,
    NextObjectValue,
    EndObject,
    ObjectKey(Cow<'a, str>),
    Eof,
    WhiteSpace(String),
}

impl<'a> JsonEvent<'a> {
    #[allow(dead_code)]
    #[allow(clippy::wrong_self_convention)]
    pub fn into_owned(&self) -> JsonEvent<'static> {
        match self {
            JsonEvent::String(s) => JsonEvent::String(Cow::Owned(s.to_string())),
            JsonEvent::Number(s) => JsonEvent::Number(Cow::Owned(s.to_string())),
            JsonEvent::Boolean(b) => JsonEvent::Boolean(*b),
            JsonEvent::Null => JsonEvent::Null,
            JsonEvent::StartArray => JsonEvent::StartArray,
            JsonEvent::EndArray => JsonEvent::EndArray,
            JsonEvent::StartObject => JsonEvent::StartObject,
            JsonEvent::NextObjectValue => JsonEvent::NextObjectValue,
            JsonEvent::EndObject => JsonEvent::EndObject,
            JsonEvent::ObjectKey(s) => JsonEvent::ObjectKey(Cow::Owned(s.to_string())),
            JsonEvent::Eof => JsonEvent::Eof,
            JsonEvent::WhiteSpace(s) => JsonEvent::WhiteSpace(s.to_string()),
            JsonEvent::NextArrayValue => JsonEvent::NextArrayValue,
        }
    }
}
