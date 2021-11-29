use std::{
    fs,
    fs::Metadata,
    io::{self, Write},
    path::{Path, PathBuf},
    process,
};

mod cli;
mod error;
mod fmt;
mod iter;

use clap::Parser;
use cli::{Algorithm, Command, CompareTrees, Opts};
use fmt::LowerHexFormatter;
use hashbrown::HashMap;
use imprint::Imprint;
use md5::Md5;
use sha1::{Digest, Sha1};
use sha2::Sha256;

type Result<T, E = error::Error> = std::result::Result<T, E>;

fn main() {
    let opts = Opts::parse();
    if let Err(e) = run(opts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(opts: Opts) -> Result<()> {
    match opts.into_command()? {
        Command::Print { path, algorithm } => display_hash(path, algorithm),
        Command::Assert {
            path,
            checksum,
            algorithm,
        } => assert(path, checksum, algorithm),
        Command::Compare { left, right } => compare(left, right),
        Command::CompareTrees(compare) => compare_trees(&compare),
    }?;

    Ok(())
}

fn assert(path: impl AsRef<Path>, expected: String, algorithm: Algorithm) -> io::Result<()> {
    let hash = match algorithm {
        Algorithm::Blake3 => hash_blake3(path)?,
        Algorithm::Md5 => hash_md5(path)?,
        Algorithm::Sha1 => hash_sha1(path)?,
        Algorithm::Sha256 => hash_sha256(path)?,
    };

    let actual = format!("{:x}", LowerHexFormatter(hash));
    let expected = expected.to_ascii_lowercase();

    if actual == expected {
        println!("True");
    } else {
        println!("False");
        process::exit(1);
    }

    Ok(())
}

fn compare<T: AsRef<Path> + Send>(left: T, right: T) -> io::Result<()> {
    use iter::IsUniform;
    use rayon::prelude::*;

    // Fun fact: hashing like this can be CPU-bound, so...
    let tasks: io::Result<Vec<_>> = [left, right].into_par_iter().map(hash_blake3).collect();

    if tasks?.uniform() {
        println!("True");
    } else {
        println!("False");
        process::exit(1);
    }

    Ok(())
}

fn compare_trees(compare: &CompareTrees) -> io::Result<()> {
    let left_tree = read_tree(compare.left.as_ref(), compare.include_hidden_files);
    let right_tree: HashMap<_, _> =
        read_tree(compare.right.as_ref(), compare.include_hidden_files).collect();

    let mut failure = false;
    for (suffix, (path, meta)) in left_tree {
        if let Some(right) = right_tree.get(&suffix) {
            if meta.len() != right.1.len() || !imprint_match(&path, &right.0)? {
                if compare.force {
                    println!("Mismatch: {}", path.display());
                    failure = true;
                } else {
                    println!("False");
                    process::exit(1);
                }
            }
        } else if compare.force {
            println!("Missing: {}", path.display());
            failure = true;
        } else {
            println!("False");
            process::exit(1);
        }
    }

    if failure {
        process::exit(1);
    }

    println!("True");
    Ok(())
}

fn display_hash(path: impl AsRef<Path>, algorithm: Algorithm) -> io::Result<()> {
    let path = path.as_ref();
    let hash = match algorithm {
        Algorithm::Blake3 => hash_blake3(path)?,
        Algorithm::Md5 => hash_md5(path)?,
        Algorithm::Sha1 => hash_sha1(path)?,
        Algorithm::Sha256 => hash_sha256(path)?,
    };

    println!("{:x}", LowerHexFormatter(hash));
    Ok(())
}

fn hash_blake3(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    hash(path.as_ref(), blake3::Hasher::new())
}

fn hash_md5(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    hash(path.as_ref(), Md5::new())
}

fn hash_sha1(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    hash(path.as_ref(), Sha1::new())
}

fn hash_sha256(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    hash(path.as_ref(), Sha256::new())
}

fn hash(path: &Path, mut digest: impl Digest + Write) -> io::Result<Vec<u8>> {
    let mut reader = fs::File::open(path).map(io::BufReader::new)?;
    io::copy(&mut reader, &mut digest)?;
    Ok(digest.finalize().as_slice().into())
}

fn read_tree(
    path: &Path,
    include_hidden_files: bool,
) -> impl Iterator<Item = (PathBuf, (PathBuf, Metadata))> + '_ {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(move |entry| {
            let entry = entry.ok()?;
            if !include_hidden_files && entry.file_name().to_string_lossy().starts_with('.') {
                return None;
            }

            if entry.file_type().is_file() {
                let meta = entry.metadata().ok()?;
                Some((PathBuf::from(entry.path()), meta))
            } else {
                None
            }
        })
        .map(move |x| (x.0.strip_prefix(path).unwrap_or(path).to_owned(), x))
}

fn imprint_match(left: &Path, right: &Path) -> io::Result<bool> {
    Ok(Imprint::new(left)? == Imprint::new(right)?)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::Opts;

    #[test]
    fn compare_files() {
        let args = &["foo", "bar.txt", "baz.txt"];
        let _opts: Opts = Parser::try_parse_from(args).unwrap();
    }
}
