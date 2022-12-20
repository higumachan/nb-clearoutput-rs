use std::borrow::Cow;

/// Possible events during JSON parsing
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum JsonEvent<'a> {
    String(Cow<'a, str>),
    Number(Cow<'a, str>),
    Boolean(bool),
    Null,
    StartArray,
    EndArray,
    StartObject,
    EndObject,
    ObjectKey(Cow<'a, str>),
    Eof,
    WhiteSpace(String),
}

impl<'a> JsonEvent<'a> {
    pub fn into_owned(&self) -> JsonEvent<'static> {
        todo!()
    }
}
