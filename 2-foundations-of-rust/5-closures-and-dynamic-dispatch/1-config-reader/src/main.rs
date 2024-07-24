use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// An imaginary config file
#[derive(Serialize, Deserialize, Debug)]
pub struct Config<'a> {
    port: u16,
    base_url: &'a str,
    s3_path: &'a str,
    database_url: &'a str,
}

#[derive(Debug)]
/// Config deserialization error
pub enum Error {
    /// Something went wrong deserializing JSON
    Json(serde_json::Error),
    /// Something went wrong deserializing YAML
    Yaml(serde_yaml::Error),
}

trait DeserializeConfig {
    /// Deserialize the contents into a `Config`
    fn deserialize<'a>(&self, contents: &'a str) -> Result<Config<'a>, Error>;
}

struct Json;
impl DeserializeConfig for Json {
    fn deserialize<'a>(&self, contents: &'a str) -> Result<Config<'a>, Error> {
        serde_json::from_str(contents).map_err(|e| Error::Json(e))
    }
}

struct Yaml;
impl DeserializeConfig for Yaml {
    fn deserialize<'a>(&self, contents: &'a str) -> Result<Config<'a>, Error> {
        serde_yaml::from_str(contents).map_err(|e| Error::Yaml(e))
    }
}

fn main() {
    let mut args = std::env::args();
    // Unwrapping is OK here, as UTF-8 Strings can always be converted to PathBufs
    let Some(path) = args.nth(1).map(|a| PathBuf::try_from(a).unwrap()) else {
        eprintln!("Please specify the input path");
        return;
    };
    // Unwrapping is Ok as `path` was created from UTF-8 string, and so is the extension
    let _extension = path.extension().map(|o| o.to_str().unwrap());
    let file_contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            // `path` was created from an UTF-8 string, so can be converted to one
            eprintln!(
                "Error reading file at path {}: {}",
                path.to_str().unwrap(),
                e
            );
            return;
        }
    };

    let deserializer: &dyn DeserializeConfig = match _extension {
        Some("json") => &Json,
        Some("yml") => &Yaml,
        Some(ext) => {
            eprintln!("Unkown file type: {}", ext);
            return;
        }
        None => {
            eprintln!("No extension");
            return;
        }
    };

    let config: Config = match deserializer.deserialize(&file_contents) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Deserialize config failed: {:?}", err);
            return;
        }
    };

    println!("Config was: {config:?}");
}
