use core::{fmt, slice};
use std::{
    borrow::Cow,
    fs, io,
    path::{Path, PathBuf},
};

use owo_colors::OwoColorize;
use uncased::AsUncased;

use crate::{alg::Algorithm, error::Error};

pub struct Hashes {
    algorithm: String,
    files: Vec<FilePair>,
}

impl Hashes {
    pub fn from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        let path = path.as_ref();
        let alg = read_alg_from_path(path)?;
        let text = fs::read_to_string(path)?;
        let entries = text.lines().filter(|&s| !s.starts_with(';'));

        let mut files = Vec::new();

        for entry in entries {
            let (hash, name) = entry
                .find(" *")
                .map(|mid| entry.split_at(mid))
                .ok_or(Error::HashFile)?;
            let name = &name[2..];

            // We have to assume the relative path here is correct -- hence the unwrap.
            let path = path.parent().unwrap().join(name);
            files.push(FilePair::new(path, hash));
        }

        Ok(Self {
            algorithm: alg.into(),
            files,
        })
    }

    pub fn exceptions(&'_ self) -> crate::Result<ExceptionsIter<'_>> {
        Ok(ExceptionsIter {
            algorithm: self.algorithm.parse()?,
            source: self.files.iter(),
        })
    }
}

fn read_alg_from_path(path: &Path) -> crate::Result<Cow<str>> {
    path.extension()
        .ok_or(Error::HashFile)
        .map(|s| s.to_string_lossy())
}

pub struct FilePair {
    path: PathBuf,
    hash: String,
}

impl FilePair {
    fn new(path: impl Into<PathBuf>, hash: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            hash: hash.into(),
        }
    }

    fn validate(&self, algorithm: Algorithm) -> io::Result<ValidationResult> {
        let actual = match algorithm.hash(&self.path) {
            Ok(actual) => actual,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(ValidationResult::Missing);
            }
            Err(e) => return Err(e),
        };

        if self.hash.as_uncased() == actual.as_uncased() {
            Ok(ValidationResult::Ok)
        } else {
            Ok(ValidationResult::Mismatch(actual))
        }
    }
}

enum ValidationResult {
    Ok,
    Mismatch(String),
    Missing,
}

pub struct ExceptionsIter<'a> {
    algorithm: Algorithm,
    source: slice::Iter<'a, FilePair>,
}

impl<'a> Iterator for ExceptionsIter<'a> {
    type Item = io::Result<Exception<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let file = self.source.next()?;
            let result = match file.validate(self.algorithm) {
                Ok(result) => result,
                Err(e) => return Some(Err(e)),
            };

            match result {
                ValidationResult::Ok => continue,
                result => return Some(Ok(Exception { file, result })),
            }
        }
    }
}

pub struct Exception<'a> {
    file: &'a FilePair,
    result: ValidationResult,
}

impl fmt::Display for Exception<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.result {
            ValidationResult::Ok => {
                let ok = "OK".bright_green();
                write!(f, "{ok} {}", self.file.path.display())
            }
            ValidationResult::Mismatch(_result) => {
                let result = "FAIL".red();
                write!(f, "{result} {}", self.file.path.display())
            }
            ValidationResult::Missing => {
                let missing = "MISSING".yellow();
                write!(f, "{missing} {}", self.file.path.display())
            }
        }
    }
}
