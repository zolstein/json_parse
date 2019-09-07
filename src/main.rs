extern crate argparse;
extern crate serde_json;
extern crate yaml_rust;

use argparse::{ArgumentParser, StoreConst, StoreOption};
use filetypes::{read_objects, FileType, ParseError};
use serde_json::Value;
use std::fs::File;
use std::io::{stdin, BufReader};

mod filetypes;

fn main() -> Result<(), ParseError> {
    let args = get_args();
    let json_in = match args.file {
        Some(f) => read_objects(BufReader::new(File::open(f)?), args.ext)?,
        None => read_objects(BufReader::new(stdin()), args.ext)?,
    };

    let content = match args.key {
        Some(key) => extract(key, json_in),
        None => Some(json_in),
    };
    match content {
        Some(content) => output(&content, &mut Vec::new()),
        None => (),
    };

    Ok(())
}

struct Args {
    key: Option<String>,
    file: Option<String>,
    ext: FileType,
}

impl Args {
    fn new() -> Args {
        Args {
            key: None,
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
            .add_option(&["-k", "--key"], StoreOption, "Key to search for");

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

fn extract(key: String, json_in: Value) -> Option<Value> {
    /*
    let pointer = JsonPointer::new(key.split('.').collect());
    return match pointer.get_owned(json_in) {
        Ok(result) => Some(result),
        _ => None,
    };
    */
    let mut current = json_in;
    for piece in key.split('.') {
        let next;
        if current.is_array() {
            let idx = match piece.parse::<usize>() {
                Ok(val) => val,
                _ => return None,
            };
            next = current.get_mut(idx)?;
        } else {
            next = current.get_mut(piece)?;
        }
        current = next.take();
    }

    Some(current)
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
        let mut name = prefix.join(".");
        if !name.is_empty() {
            name.push(':');
        }
        println!("{}{}", name, value);
    }
}
