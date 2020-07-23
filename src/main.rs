mod fmt;
mod iter;

use fmt::LowerHexFormatter;
use hashbrown::HashMap;
use imprint::Imprint;
use sha2::{Digest, Sha256};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::{fs, io, process};

enum Command {
    Print {
        path: String,
    },
    Assert {
        path: String,
        checksum: String,
    },
    Compare {
        left: String,
        right: String,
    },
    CompareTrees {
        left: String,
        right: String,
        force: bool,
    },
}

fn read_command() -> Command {
    use clap::{load_yaml, App, AppSettings};

    let yaml = load_yaml!("../args.yaml");
    let args = App::from_yaml(yaml)
        .global_setting(AppSettings::SubcommandsNegateReqs)
        .get_matches();

    if let Some(sub) = args.subcommand_matches("assert") {
        return Command::Assert {
            path: sub.value_of("path").unwrap().to_string(),
            checksum: sub.value_of("checksum").unwrap().to_string(),
        };
    }

    if let Some(sub) = args.subcommand_matches("compare") {
        return Command::Compare {
            left: sub.value_of("left").unwrap().to_string(),
            right: sub.value_of("right").unwrap().to_string(),
        };
    }

    if let Some(sub) = args.subcommand_matches("compare-trees") {
        return Command::CompareTrees {
            left: sub.value_of("left").unwrap().to_string(),
            right: sub.value_of("right").unwrap().to_string(),
            force: sub.is_present("force"),
        };
    }

    Command::Print {
        path: args.value_of("path").unwrap().to_string(),
    }
}

fn main() -> io::Result<()> {
    match read_command() {
        Command::Print { path } => display_hash(path),
        Command::Assert { path, checksum } => assert(path, checksum),
        Command::Compare { left, right } => compare(left, right),
        Command::CompareTrees { left, right, force } => compare_trees(left, right, force),
    }
}

fn assert(path: impl AsRef<Path>, expected: String) -> io::Result<()> {
    let actual = format!("{:x}", LowerHexFormatter(hash(path)?));
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
    let tasks: io::Result<Vec<_>> = [left, right].into_par_iter().map(hash).collect();

    if tasks?.is_uniform() {
        println!("True");
    } else {
        println!("False");
        process::exit(1);
    }

    Ok(())
}

fn compare_trees<T: AsRef<Path> + Send>(left: T, right: T, force: bool) -> io::Result<()> {
    let left_tree = read_tree(left.as_ref());
    let right_tree: HashMap<_, _> = read_tree(right.as_ref()).collect();

    let mut failure = false;
    for (suffix, (path, meta)) in left_tree {
        if let Some(right) = right_tree.get(&suffix) {
            if meta.len() != right.1.len() || !imprint_match(&path, &right.0)? {
                if force {
                    println!("Mismatch: {}", path.display());
                    failure = true;
                } else {
                    println!("False");
                    process::exit(1);
                }
            }
        } else if force {
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

fn display_hash(path: impl AsRef<Path>) -> io::Result<()> {
    println!("{:x}", LowerHexFormatter(hash(path)?));
    Ok(())
}

fn hash(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut hasher = Sha256::new();
    let mut reader = fs::File::open(path).map(io::BufReader::new)?;
    io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize().as_slice().into())
}

fn read_tree<'a>(path: &'a Path) -> impl Iterator<Item = (PathBuf, (PathBuf, Metadata))> + 'a {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
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
