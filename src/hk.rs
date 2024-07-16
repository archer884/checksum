use core::{fmt, slice};
use std::{
    borrow::Cow,
    fs, io,
    path::{Path, PathBuf},
};

use owo_colors::OwoColorize;
use regex::Regex;
use uncased::AsUncased;

use crate::{alg::Algorithm, error::Error};

pub struct Hashes {
    algorithm: Algorithm,
    files: Vec<ValidateTask>,
}

impl Hashes {
    pub fn from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        let path = path.as_ref();
        let algorithm = read_alg_from_path(path)?.parse()?;
        let text = fs::read_to_string(path)?;
        let entries = text.lines().filter(|&s| !s.starts_with('#'));
        let parser = EntryParser::default();

        let mut files = Vec::new();

        // This should work with or without asterisks.
        // ref: https://www.howtogeek.com/67241/htg-explains-what-are-md5-sha-1-hashes-and-how-do-i-check-them/
        for entry in entries {
            let (hash, name) = parser.parse(entry)?;

            // We have to assume the relative path here is correct -- hence the unwrap.
            let path = path.parent().expect("path must refer to file").join(name);
            files.push(ValidateTask::new(path, name, hash));
        }

        Ok(Self { algorithm, files })
    }

    /// If you don't use this iterator, nothing actually gets verified.
    #[must_use]
    pub fn verify(&'_ self) -> Validator<'_> {
        Validator {
            algorithm: self.algorithm,
            source: self.files.iter(),
        }
    }
}

#[derive(Debug)]
struct EntryParser {
    rx: Regex,
}

impl EntryParser {
    fn parse<'a>(&self, entry: &'a str) -> crate::Result<(&'a str, &'a str)> {
        let cx = self.rx.captures(entry).ok_or(Error::HashFile)?;
        let hash = cx.get(1).ok_or(Error::HashFile)?.as_str();
        let name = cx.get(2).ok_or(Error::HashFile)?.as_str();
        Ok((hash, name))
    }
}

impl Default for EntryParser {
    fn default() -> Self {
        Self {
            rx: Regex::new(r"^(\S+)\s+\*?(.+)$").unwrap(),
        }
    }
}

fn read_alg_from_path(path: &Path) -> crate::Result<Cow<str>> {
    path.extension()
        .ok_or(Error::HashFile)
        .map(|s| s.to_string_lossy())
}

pub struct ValidateTask {
    path: PathBuf,
    name: String,
    hash: String,
}

impl ValidateTask {
    fn new(path: impl Into<PathBuf>, name: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            hash: hash.into(),
        }
    }

    fn validate(&self, algorithm: Algorithm) -> io::Result<HashResult> {
        let actual = match algorithm.hash(&self.path) {
            Ok(actual) => actual,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(HashResult::Missing);
            }
            Err(e) => return Err(e),
        };

        if self.hash.as_uncased() == actual.as_uncased() {
            Ok(HashResult::Ok)
        } else {
            Ok(HashResult::Mismatch(actual))
        }
    }
}

enum HashResult {
    Ok,
    Mismatch(String),
    Missing,
}

pub struct Validator<'a> {
    algorithm: Algorithm,
    source: slice::Iter<'a, ValidateTask>,
}

impl<'a> Iterator for Validator<'a> {
    type Item = io::Result<Validation<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let file = self.source.next()?;
        let result = match file.validate(self.algorithm) {
            Ok(result) => result,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok(Validation { file, result }))
    }
}

pub struct Validation<'a> {
    file: &'a ValidateTask,
    result: HashResult,
}

impl fmt::Display for Validation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.result {
            HashResult::Ok => {
                let ok = "OK".bright_green();
                write!(f, "{ok} {}", self.file.name)
            }
            HashResult::Mismatch(_result) => {
                let result = "FAIL".red();
                write!(f, "{result} {}", self.file.name)
            }
            HashResult::Missing => {
                let missing = "MISSING".yellow();
                write!(f, "{missing} {}", self.file.name)
            }
        }
    }
}
