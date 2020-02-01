extern crate argparse;
extern crate json_pointer;
extern crate serde_json;
extern crate yaml_rust;

use argparse::{ArgumentParser, Store, StoreConst, StoreOption};
use filetypes::{read_objects, FileType, ReadError};
use json_pointer::JsonPointer;
use serde_json::Value;
use std::fs::File;
use std::io::{self, stdin, BufReader};

mod filetypes;

#[derive(Debug)]
enum RunError {
    IoError(io::Error),
    ObjectError(ReadError),
}

impl From<io::Error> for RunError {
    fn from(e: io::Error) -> RunError {
        RunError::IoError(e)
    }
}

impl From<ReadError> for RunError {
    fn from(e: ReadError) -> RunError {
        match e {
            ReadError::IoError(ioe) => RunError::IoError(ioe),
            _ => RunError::ObjectError(e),
        }
    }
}

fn main() -> Result<(), RunError> {
    let args = get_args();
    let json_in = match args.file {
        Some(f) => read_objects(BufReader::new(File::open(f)?), args.ext)?,
        None => read_objects(BufReader::new(stdin()), args.ext)?,
    };

    let content = extract(&args.key, &json_in);
    content.map(|c| output(c, &mut Vec::new()));

    Ok(())
}

struct Args {
    key: String,
    file: Option<String>,
    ext: FileType,
}

impl Args {
    fn new() -> Args {
        Args {
            key: String::from(""),
            file: None,
            ext: FileType::Json,
        }
    }
}

fn get_args() -> Args {
    let mut args = Args::new();
    let mut ext: Option<FileType> = None;
    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut args.key)
            .add_option(&["-k", "--key"], Store, "Key to search for");

        parser.refer(&mut args.file).add_option(
            &["-f", "--file"],
            StoreOption,
            "File to read from (default stdin)",
        );

        parser
            .refer(&mut ext)
            .add_option(
                &["--yaml"],
                StoreConst(Some(FileType::Yaml)),
                "Parse input as yaml",
            )
            .add_option(
                &["--json"],
                StoreConst(Some(FileType::Json)),
                "Parse input as json",
            );

        parser.parse_args_or_exit();
    }

    let filename: Option<&String> = Option::from(&args.file);
    args.ext = ext
        .or_else(|| {
            filename
                .map(|x| get_extension(&x))
                .map(FileType::for_extension)?
        })
        .unwrap_or(FileType::Json);

    args
}

fn get_extension(filename: &str) -> &str {
    filename.rsplitn(2, '.').next().unwrap()
}

fn extract<'a>(pointer: &String, json_in: &'a Value) -> Option<&'a Value> {
    Some(json_in.pointer(pointer)?)
}

fn output(value: &Value, prefix: &mut Vec<String>) {
    if value.is_array() {
        let array = value.as_array().unwrap();
        for (i, v) in array.into_iter().enumerate() {
            prefix.push(i.to_string());
            output(v, prefix);
            prefix.pop();
        }
    } else if value.is_object() {
        let obj = value.as_object().unwrap();
        for (k, v) in obj {
            prefix.push(k.to_string());
            output(v, prefix);
            prefix.pop();
        }
    } else {
        let mut name = JsonPointer::new(prefix).to_string();
        if !name.is_empty() {
            name.push(':');
        }
        println!("{}{}", name, value);
    }
}
