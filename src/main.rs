mod fmt;
mod iter;

use fmt::LowerHexFormatter;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::{fs, io};
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
struct Opt {
    /// A file path.
    path: String,
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Clone, Debug, StructOpt)]
enum Command {
    /// Test a file against a provided checksum.
    Assert {
        /// The checksum to be asserted.
        expected: String,
    },

    /// Test a file against another file.
    Eq {
        /// A file path to compare against.
        other_path: String,
    },
}

fn main() -> io::Result<()> {
    let Opt { path, cmd } = Opt::from_args();
    match cmd {
        None => display_hash(path),
        Some(Command::Assert { expected }) => assert(path, expected),
        Some(Command::Eq { other_path }) => compare(path, other_path),
    }
}

fn assert(path: impl AsRef<Path>, expected: String) -> io::Result<()> {
    let actual = format!("{:x}", LowerHexFormatter(hash(path)?));
    let expected = expected.to_ascii_lowercase();

    if actual == expected {
        println!("True");
    } else {
        println!("False");
    }
    Ok(())
}

fn compare<T: AsRef<Path> + Send>(left: T, right: T) -> io::Result<()> {
    use iter::CompareAgainstHead;
    use rayon::prelude::*;

    // Fun fact: hashing like this can be CPU-bound, so...
    let tasks: io::Result<Vec<_>> = [left, right].into_par_iter().map(hash).collect();

    if tasks?.all_items_match() {
        println!("True");
    } else {
        println!("False");
    }

    Ok(())
}

fn display_hash(path: impl AsRef<Path>) -> io::Result<()> {
    println!("{:x}", LowerHexFormatter(hash(path)?));
    Ok(())
}

fn hash(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut hasher = Sha256::new();
    let mut reader = fs::File::open(path).map(io::BufReader::new)?;

    io::copy(&mut reader, &mut hasher)?;

    Ok(Vec::from(hasher.result().as_slice()))
}
