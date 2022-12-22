#![allow(dead_code)]

use crate::event::JsonEvent;
use anyhow::Result;
use std::io::Write;

pub struct JsonWriter<W: Write> {
    writer: W,
}

impl<W: Write> JsonWriter<W> {
    pub fn from_writer(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_event(&mut self, event: JsonEvent) -> Result<()> {
        match event {
            JsonEvent::WhiteSpace(whitespace) => {
                self.writer.write_all(whitespace.as_bytes())?;
            }
            JsonEvent::String(string) => {
                self.writer.write_all(b"\"")?;
                self.writer.write_all(string.as_bytes())?;
                self.writer.write_all(b"\"")?;
            }
            JsonEvent::Number(number) => {
                self.writer.write_all(number.as_bytes())?;
            }
            JsonEvent::Boolean(boolean) => {
                self.writer
                    .write_all(if boolean { b"true" } else { b"false" })?;
            }
            JsonEvent::Null => {
                self.writer.write_all(b"null")?;
            }
            JsonEvent::StartObject => {
                self.writer.write_all(b"{")?;
            }
            JsonEvent::NextObjectValue => {
                self.writer.write_all(b",")?;
            }
            JsonEvent::EndObject => {
                self.writer.write_all(b"}")?;
            }
            JsonEvent::ObjectKey(key) => {
                self.writer.write_all(b"\"")?;
                self.writer.write_all(key.as_bytes())?;
                self.writer.write_all(b"\":")?;
            }
            JsonEvent::StartArray => {
                self.writer.write_all(b"[")?;
            }
            JsonEvent::NextArrayValue => {
                self.writer.write_all(b",")?;
            }
            JsonEvent::EndArray => {
                self.writer.write_all(b"]")?;
            }
            JsonEvent::Eof => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read::JsonReader;
    use rstest::rstest;
    use std::borrow::Cow;
    use std::io::{BufReader, Cursor};

    fn write_events(events: Vec<JsonEvent>) -> String {
        let mut buffer = Vec::new();
        {
            let mut writer = JsonWriter::from_writer(&mut buffer);
            for event in events {
                writer.write_all_event(event).unwrap();
            }
        }

        String::from_utf8(buffer).unwrap()
    }

    #[test]
    fn simple_write() {
        let events = vec![
            JsonEvent::StartObject,
            JsonEvent::ObjectKey(Cow::Owned("key".to_string())),
            JsonEvent::String(Cow::Owned("value".to_string())),
            JsonEvent::EndObject,
        ];

        let json_str = write_events(events);
        assert_eq!(json_str, "{\"key\":\"value\"}");
    }

    #[test]
    fn with_whitespace() {
        let events = vec![
            JsonEvent::WhiteSpace("\n  ".to_string()),
            JsonEvent::StartObject,
            JsonEvent::ObjectKey(Cow::Owned("key".to_string())),
            JsonEvent::String(Cow::Owned("value".to_string())),
            JsonEvent::EndObject,
        ];

        let json_str = write_events(events);
        assert_eq!(json_str, "\n  {\"key\":\"value\"}");
    }

    #[rstest]
    fn read_and_write(
        #[values(
            "        {\"key\":    \"value\"  \n, \"key2\": 123}   ",
            "    [ 1 , 2 , 3]   "
        )]
        json_str: &str,
    ) {
        let mut reader = JsonReader::from_reader(BufReader::new(Cursor::new(json_str.as_bytes())));
        let mut output_json_buffer = vec![];
        {
            let mut writer = JsonWriter::from_writer(&mut output_json_buffer);

            let mut buffer = vec![];
            loop {
                let event = reader.read_event(&mut buffer).unwrap();
                dbg!(&event);
                if event == JsonEvent::Eof {
                    break;
                }
                writer.write_all_event(event).unwrap();
            }
        }
        let output_json_str = String::from_utf8(output_json_buffer).unwrap();

        assert_eq!(output_json_str, json_str);
    }
}
