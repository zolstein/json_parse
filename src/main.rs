extern crate argparse;
extern crate serde_json;

use std::io::{self, stdin, BufReader};
use std::fs::File;
use argparse::{ArgumentParser, StoreOption};
use serde_json::{Value};

fn main() -> io::Result<()> {
    let (key, file) = get_args();
    let json_in = match file {
        Some(f) => serde_json::from_reader(BufReader::new(File::open(f)?))?,
        None => serde_json::from_reader(BufReader::new(stdin()))?,
    };

    let content = match key {
        Some(key) => extract(key, json_in),
        None => Some(json_in),
    };
    match content {
        Some(content) => output(&content, &mut Vec::new()),
        None => (),
    };

    Ok(())
}

fn get_args() -> (Option<String>, Option<String>) {
    let mut key = None;
    let mut file = None;
    {
        let mut parser = ArgumentParser::new();
        parser.refer(&mut key)
            .add_option(&["-k", "--key"], StoreOption, "Key to search for");
        parser.refer(&mut file)
            .add_option(&["-f", "--file"], StoreOption, "File to read from");
        parser.parse_args_or_exit();
    }

    (key, file)
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
    // println!("Content: {}, Prefix: {}", value, prefix.join("."));
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
