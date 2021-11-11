use std::{fmt::Display, io, rc::Rc};

#[derive(Copy, Clone, Debug)]
pub enum Entry {
    Dir,
    File,
}

#[derive(Clone, Debug)]
pub enum Error {
    // We only store the first object; the second object is assumed to be
    // A) wrong and B) the opposite kind.
    IllegalComparison(Entry),
    Io(Rc<io::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IllegalComparison(entry) => match entry {
                Entry::Dir => f.write_str("attempt to compare dir against file"),
                Entry::File => f.write_str("attempt to compare file against dir"),
            },
            Error::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Self::Io(Rc::new(v))
    }
}
