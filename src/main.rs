use clap::Parser;
use json_event_parser::{JsonEvent, JsonReader, JsonWriter};
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
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!(
        "Hello, world! {:?} {}",
        args.output,
        args.input_file.to_string_lossy()
    );

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

    loop {
        let event: JsonEvent = json_reader.read_event(&mut buffer)?;

        if event == JsonEvent::Eof {
            break;
        }

        if let (State::OutputsArrayStart, JsonEvent::EndArray) = (state, &event) {
            stack -= 1;
            if stack == 0 {
                state = State::OutputsArrayEnd;
                skip = false;
            }
        }

        if !skip {
            writer.write_event(event.clone())?;
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
