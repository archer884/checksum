use std::path::Path;

use clap::Parser;

use crate::error::{Entry, Error};

#[derive(Clone, Debug, Parser)]
#[clap(version = clap::crate_version!())]
pub struct Opts {
    /// a file to be hashed
    path: String,

    /// a file to compare against
    #[clap(group = "resource")]
    compare: Option<String>,

    #[clap(flatten)]
    hashing: Hashing,

    /// when comparing trees, force a full comparison and list exceptions
    #[clap(short, long)]
    force: bool,

    /// when comparing trees, include hidden files
    #[clap(short, long)]
    hidden: bool,
}

impl Opts {
    pub fn into_command(self) -> crate::Result<Command> {
        let path = self.path;

        // If we have a comparison path, it needs to match the type of the primary path.
        if let Some(compare) = self.compare {
            let left = Path::new(&path);
            let right = Path::new(&compare);

            let left_is_dir = left.is_dir();

            if left_is_dir && right.is_dir() {
                return Ok(Command::CompareTrees(CompareTrees {
                    left: path,
                    right: compare,
                    force: self.force,
                    include_hidden_files: self.hidden,
                }));
            }

            if left.is_file() && right.is_file() {
                return Ok(Command::Compare {
                    left: path,
                    right: compare,
                });
            }

            return Err(Error::IllegalComparison(if left.is_file() {
                Entry::File
            } else {
                Entry::Dir
            }));
        }

        let (algorithm, checksum) = self.hashing.get_algorithm();

        // If we have a checksum, this is an assert.
        if let Some(checksum) = checksum {
            return Ok(Command::Assert {
                path,
                checksum,
                algorithm,
            });
        }

        // Otherwise, we're just going to hash the file and print the checksum.
        Ok(Command::Print { path, algorithm })
    }
}

#[derive(Clone, Debug, Parser)]
struct Hashing {
    #[clap(
        short,
        long,
        long_about = "set blake3 mode and supply an (optional) checksum for comparison"
    )]
    blake3: Option<Option<String>>,

    #[clap(
        short,
        long,
        group = "resource",
        long_about = "set md5 mode and supply an (optional) checksum for comparison"
    )]
    md5: Option<Option<String>>,

    #[clap(
        short = 'd',
        long,
        group = "resource",
        long_about = "set sha1 mode and supply an (optional) checksum for comparison"
    )]
    sha1: Option<Option<String>>,

    #[clap(
        short = 's',
        long,
        group = "resource",
        long_about = "set sha256 mode and supply an (optional) checksum for comparison"
    )]
    sha256: Option<Option<String>>,

    #[clap(
        short = 'S',
        long,
        group = "resource",
        long_about = "set sha512 mode and supply an (optional) checksum for comparison"
    )]
    sha512: Option<Option<String>>,
}

impl Hashing {
    fn get_algorithm(self) -> (Algorithm, Option<String>) {
        if let Some(blake3) = self.blake3 {
            (Algorithm::Blake3, blake3)
        } else if let Some(md5) = self.md5 {
            (Algorithm::Md5, md5)
        } else if let Some(sha1) = self.sha1 {
            (Algorithm::Sha1, sha1)
        } else if let Some(sha256) = self.sha256 {
            (Algorithm::Sha256, sha256)
        } else if let Some(sha512) = self.sha512 {
            (Algorithm::Sha512, sha512)
        } else {
            // Seems like sha1 is pretty much still the most popular for this
            (Algorithm::Sha1, None)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Algorithm {
    Blake3,
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Clone, Debug)]
pub struct CompareTrees {
    pub left: String,
    pub right: String,
    pub force: bool,
    pub include_hidden_files: bool,
}

pub enum Command {
    Print {
        path: String,
        algorithm: Algorithm,
    },
    Assert {
        path: String,
        checksum: String,
        algorithm: Algorithm,
    },
    Compare {
        left: String,
        right: String,
    },
    CompareTrees(CompareTrees),
}
