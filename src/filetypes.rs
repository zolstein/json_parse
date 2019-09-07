use serde_json::{map, Value};
use std::collections::HashMap;
use std::io::{self, Read};
use std::result;
use yaml_rust::{scanner, yaml, Yaml, YamlLoader};

type Result<T> = result::Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    ScanError(scanner::ScanError),
    JsonError(serde_json::Error),
    JsonConvertError(String),
}

impl ParseError {
    fn conversion_error(msg: String) -> ParseError {
        ParseError::JsonConvertError(msg)
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::IoError(err)
    }
}

impl From<scanner::ScanError> for ParseError {
    fn from(err: scanner::ScanError) -> ParseError {
        ParseError::ScanError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> ParseError {
        ParseError::JsonError(err)
    }
}

#[derive(Copy, Clone)]
pub enum FileType {
    Json,
    Yaml,
}

pub fn read_objects<R: Read>(reader: R, filetype: FileType) -> Result<Value> {
    match filetype {
        FileType::Json => read_from_json(reader),
        FileType::Yaml => read_from_yaml(reader),
    }
}

impl FileType {
    pub fn for_extension(ext: &str) -> Option<FileType> {
        match ext {
            "yaml" => Some(FileType::Yaml),
            "yml" => Some(FileType::Yaml),
            "json" => Some(FileType::Json),
            _ => None,
        }
    }
}

fn read_from_json<R: Read>(reader: R) -> Result<Value> {
    Ok(serde_json::from_reader(reader)?)
}

fn read_from_yaml<R: Read>(mut reader: R) -> Result<Value> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;
    let mut doc = YamlLoader::load_from_str(&buffer)?;
    return yaml_to_json(doc.remove(0));
}

fn yaml_to_json(yaml: Yaml) -> Result<Value> {
    match yaml {
        Yaml::Hash(h) => handle_hash(h),
        Yaml::Array(a) => handle_array(a),
        Yaml::String(s) => Ok(s.into()),
        Yaml::Integer(i) => Ok(i.into()),
        Yaml::Real(r) => Ok(r.into()), // Sort of a hack, since we will print as string
        Yaml::Boolean(b) => Ok(b.into()),
        Yaml::Null => Ok(Value::Null),
        _ => Err(ParseError::conversion_error(
            "Unsupported value type".to_string(),
        )),
    }
}

fn handle_hash(hash: yaml::Hash) -> Result<Value> {
    let mut result: HashMap<String, Value> = HashMap::new();
    for (k, v) in hash.into_iter() {
        let key = yaml_to_key_string(k)?;
        let value = yaml_to_json(v)?;
        result.insert(key, value);
    }
    Ok(result
        .into_iter()
        .collect::<map::Map<String, Value>>()
        .into())
}

fn handle_array(array: yaml::Array) -> Result<Value> {
    let result: Vec<Value> = array
        .into_iter()
        .map(yaml_to_json)
        .collect::<Result<Vec<Value>>>()?;
    Ok(result.into())
}

fn yaml_to_key_string(yaml: Yaml) -> Result<String> {
    match yaml {
        Yaml::String(s) => Ok(s),
        Yaml::Integer(i) => Ok(i.to_string()),
        Yaml::Real(s) => Ok(s),
        Yaml::Boolean(b) => Ok(b.to_string()),
        _ => Err(ParseError::conversion_error(
            "Non-stringable key".to_string(),
        )),
    }
}
