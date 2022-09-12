use std::{
    fs,
    iter::Product,
    path::{Path, PathBuf},
};

use regex::{Captures, Regex};

pub struct WgslPreProcessor {
    root: PathBuf,
}

#[derive(Debug)]
pub enum PreProcessError {
    FileNotFound(String),
    FileReadError(String),
    Other(String),
}

impl WgslPreProcessor {
    pub fn new(root: &str) -> WgslPreProcessor {
        WgslPreProcessor {
            root: PathBuf::from(root),
        }
    }
    pub fn preprocess(&self, src: &str) -> Result<String, PreProcessError> {
        let reg = r#"\#include\("(.+.wgsl)"\);"#;
        let re = Regex::new(reg).unwrap();
        let mut processed = src.to_owned();
        for cap in re.captures(src) {
            let filename = cap[1].to_owned();
            let file_path = self.root.join(filename);
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

    pub fn load_and_process(&self, file: &str) -> Result<String, PreProcessError> {
        match fs::read_to_string(self.root.join(file)) {
            Ok(contents) => self.preprocess(contents.as_str()),
            Err(e) => Err(PreProcessError::FileNotFound(format!(
                "Couldn't read file: {}",
                file.to_owned()
            ))),
        }
    }
}
