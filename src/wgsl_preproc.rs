use std::{
    fs,
    iter::Product,
    path::{Path, PathBuf},
};

use regex::{Captures, Regex};

pub struct WgslPreProcessor {
    root: PathBuf,
}

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
    pub fn preprocess(&self, src: &str) -> String {
        let reg = r#"\#include\("(.+.wgsl)"\);"#;
        let re = Regex::new(reg).unwrap();

        let processed = re.replace_all(src, |caps: &Captures| {
            let filename = caps[1].to_owned();
            let file_path = self.root.join(filename);

            let contents = fs::read_to_string(file_path).unwrap();
            contents
        });
        processed.to_string()
    }

    pub fn load_and_process(&self, file: &str) -> String {
        let contents = fs::read_to_string(self.root.join(file)).unwrap();
        self.preprocess(contents.as_str())
    }
}
