#![allow(dead_code)]

use crate::event::JsonEvent;
use anyhow::Result;
use std::borrow::Borrow;
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
                write_escaped_json_string(string.borrow(), &mut self.writer)?;
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
                write_escaped_json_string(key.borrow(), &mut self.writer)?;
                self.writer.write_all(b":")?;
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

fn write_escaped_json_string(s: &str, sink: &mut impl Write) -> Result<()> {
    sink.write_all(b"\"")?;
    let mut buffer = [b'\\', b'u', 0, 0, 0, 0];
    for c in s.chars() {
        match c {
            '\\' => sink.write_all(b"\\\\"),
            '"' => sink.write_all(b"\\\""),
            c => {
                if c < char::from(32) {
                    match c {
                        '\u{08}' => sink.write_all(b"\\b"),
                        '\u{0C}' => sink.write_all(b"\\f"),
                        '\n' => sink.write_all(b"\\n"),
                        '\r' => sink.write_all(b"\\r"),
                        '\t' => sink.write_all(b"\\t"),
                        c => {
                            let mut c = c as u8;
                            for i in (2..6).rev() {
                                let ch = c % 16;
                                buffer[i] = ch + if ch < 10 { b'0' } else { b'A' };
                                c /= 16;
                            }
                            sink.write_all(&buffer)
                        }
                    }
                } else {
                    sink.write_all(c.encode_utf8(&mut buffer[2..]).as_bytes())
                }
            }
        }?;
    }
    sink.write_all(b"\"")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read::JsonReader;
    use rstest::rstest;
    use std::borrow::Cow;
    use std::fs::File;
    use std::io::{BufReader, Cursor, Read};

    fn write_events(events: Vec<JsonEvent>) -> String {
        let mut buffer = Vec::new();
        {
            let mut writer = JsonWriter::from_writer(&mut buffer);
            for event in events {
                writer.write_event(event).unwrap();
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
                writer.write_event(event).unwrap();
            }
        }
        let output_json_str = String::from_utf8(output_json_buffer).unwrap();

        assert_eq!(output_json_str, json_str);
    }

    #[test]
    fn read_and_write_realcase() {
        let mut buf = String::new();
        File::open("assets/notebook/sample.ipynb")
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();

        let mut reader = JsonReader::from_reader(BufReader::new(Cursor::new(buf.as_bytes())));
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
                writer.write_event(event).unwrap();
            }
        }
        let output_json_str = String::from_utf8(output_json_buffer).unwrap();

        assert_eq!(output_json_str, buf);
    }
}
