use std::{fmt::Display, io, rc::Rc};

#[derive(Copy, Clone, Debug)]
pub enum OperationKind {
    Child,
    Dir,
    File,
}

#[derive(Clone, Debug)]
pub enum Error {
    // We only store the first object; the second object is assumed to be
    // A) wrong and B) the opposite kind.
    InvalidOperation(OperationKind),
    Io(Rc<io::Error>),
    UnknownAlgorithm(String),
    HashFile,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidOperation(entry) => match entry {
                OperationKind::Child => f.write_str("attempt to compare dir against parent dir"),
                OperationKind::Dir => f.write_str("cannot compare directory against non-directory"),
                OperationKind::File => f.write_str("cannot compare file against non-file"),
            },
            Error::Io(e) => e.fmt(f),
            Error::UnknownAlgorithm(algorithm) => write!(f, "unknown algorithm: {algorithm}"),
            Error::HashFile => f.write_str("bad hash file format"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Self::Io(Rc::new(v))
    }
}
