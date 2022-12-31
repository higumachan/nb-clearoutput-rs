use clap::Parser;
use json_event_parser_witespace::{JsonEvent, JsonReader, JsonWriter};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(value_enum)]
    output: Output,
    input_file: PathBuf,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum Output {
    Inplace,
    Stdout,
}

#[derive(Debug, Clone, Copy)]
enum State {
    Root,
    Cells,
    CellsArrayStart,
    Outputs,
    OutputsArrayStart,
    OutputsArrayEnd,
    ExecutionCount,
    MetaData,
    Collapsed,
}

fn flush_events<'a>(
    skip: bool,
    writer: &mut JsonWriter<Box<dyn Write>>,
    spaces: &mut Vec<String>,
    event: &JsonEvent<'a>,
) -> anyhow::Result<()> {
    if !skip {
        for space in std::mem::take(spaces) {
            writer.write_event(JsonEvent::WhiteSpace(space))?;
        }
        writer.write_event(event.to_owned())?;
    } else {
        spaces.clear();
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file_reader = BufReader::new(File::open(&args.input_file)?);
    let mut json_reader: JsonReader<BufReader<File>> = JsonReader::from_reader(file_reader);
    let mut buffer = Vec::new();

    let mut writer: JsonWriter<Box<dyn Write>> = JsonWriter::from_writer(match args.output {
        Output::Inplace => Box::new(BufWriter::new(File::create(&args.input_file)?)),
        Output::Stdout => Box::new(BufWriter::new(std::io::stdout())),
    });

    let mut state = State::Root;
    let mut stack = 0;
    let mut skip = false;
    let mut save_spaces = vec![];

    loop {
        let event: JsonEvent = json_reader.read_event(&mut buffer)?.to_owned();

        if event == JsonEvent::Eof {
            break;
        }

        match (state, &event) {
            (State::OutputsArrayStart, JsonEvent::EndArray) => {
                stack -= 1;
                if stack == 0 {
                    state = State::OutputsArrayEnd;
                    skip = false;
                }
                save_spaces.clear();
                flush_events(skip, &mut writer, &mut save_spaces, &event)?;
            }
            (State::ExecutionCount, JsonEvent::Number(_) | JsonEvent::Null) => {
                state = State::OutputsArrayEnd;
                skip = false;
                flush_events(skip, &mut writer, &mut save_spaces, &JsonEvent::Null)?;
            }
            (State::MetaData, JsonEvent::EndObject) => {
                state = State::OutputsArrayEnd;
                flush_events(skip, &mut writer, &mut save_spaces, &event)?;
            }
            (State::Collapsed, JsonEvent::NextObjectValue) => {
                state = State::MetaData;
                skip = false;
            }
            (State::MetaData, JsonEvent::ObjectKey(key)) if key == "collapsed" => {
                state = State::Collapsed;
                skip = true;
            }
            (_, JsonEvent::WhiteSpace(space)) => {
                save_spaces.push(space.clone());
            }
            _ => {
                flush_events(skip, &mut writer, &mut save_spaces, &event)?;
            }
        }

        match (state, event) {
            (State::Root, JsonEvent::ObjectKey(key)) if key == "cells" => {
                state = State::Cells;
            }
            (State::Cells, JsonEvent::StartArray) => {
                state = State::CellsArrayStart;
            }
            (State::CellsArrayStart | State::OutputsArrayEnd, JsonEvent::ObjectKey(key))
                if key == "outputs" =>
            {
                state = State::Outputs;
            }
            (State::CellsArrayStart | State::OutputsArrayEnd, JsonEvent::ObjectKey(key))
                if key == "execution_count" =>
            {
                state = State::ExecutionCount;
                skip = true;
            }
            (State::CellsArrayStart | State::OutputsArrayEnd, JsonEvent::ObjectKey(key))
                if key == "metadata" =>
            {
                state = State::MetaData;
            }
            (State::Outputs, JsonEvent::StartArray) => {
                state = State::OutputsArrayStart;
                skip = true;
                stack += 1;
            }
            (State::OutputsArrayStart, JsonEvent::StartArray) => {
                stack += 1;
            }
            _ => {}
        }
    }

    Ok(())
}
