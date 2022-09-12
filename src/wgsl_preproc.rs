use std::{fs, path::PathBuf, str::FromStr};

use regex::Regex;
pub struct WgslPreProcessor {}
#[derive(Debug)]
pub enum PreProcessError {
    FileNotFound(String),
    FileReadError(String),
    Other(String),
}

impl WgslPreProcessor {
    pub fn preprocess(src: &str, shader_root: &str) -> Result<String, PreProcessError> {
        let root = PathBuf::from_str(shader_root).unwrap();
        let reg = r#"\#include\("(.+.wgsl)"\);"#;
        let re = Regex::new(reg).unwrap();
        let mut processed = src.to_owned();
        for cap in re.captures(src) {
            let filename = cap[1].to_owned();
            let file_path = root.join(filename);
            let repl = cap[0].to_owned();
            if (!file_path.exists()) {
                return Err(PreProcessError::FileNotFound(format!(
                    "Couldn't find file: {}",
                    cap[1].to_owned()
                )));
            }
            match fs::read_to_string(file_path) {
                Ok(contents) => processed = processed.replace(&repl, &contents),
                Err(_) => {
                    return Err(PreProcessError::FileReadError(format!(
                        "Couldn't read file: {}",
                        cap[1].to_owned()
                    )))
                }
            }
        }
        //processed.to_string()
        Ok(processed)
    }

    pub fn load_and_process(file: &str, shader_root: &str) -> Result<String, PreProcessError> {
        let root = PathBuf::from_str(shader_root).unwrap();
        match fs::read_to_string(root.join(file)) {
            Ok(contents) => WgslPreProcessor::preprocess(contents.as_str(), shader_root),
            Err(e) => Err(PreProcessError::FileNotFound(format!(
                "Couldn't read file: {}",
                file.to_owned()
            ))),
        }
    }
}
