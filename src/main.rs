mod fmt;
mod iter;

use fmt::LowerHexFormatter;
use imprint::Imprint;
use sha2::{Digest, Sha256};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::{fs, io, process};

enum Command {
    Print { path: String },
    Assert { path: String, checksum: String },
    Compare { left: String, right: String },
    CompareTrees { left: String, right: String },
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
        Command::CompareTrees { left, right } => compare_trees(left, right),
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

fn compare_trees<T: AsRef<Path> + Send>(left: T, right: T) -> io::Result<()> {
    let left_tree = read_tree(left.as_ref());
    let right_tree = read_tree(right.as_ref());

    for (l, r) in left_tree.zip(right_tree) {
        let lp = l.0.strip_prefix(left.as_ref()).unwrap_or(&l.0);
        let rp = r.0.strip_prefix(right.as_ref()).unwrap_or(&r.0);

        if lp != rp || l.1.len() != r.1.len() || !imprint_mismatch(&l.0, &r.0)? {
            println!("False");
            process::exit(1);
        }
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
    Ok(Vec::from(hasher.result().as_slice()))
}

fn read_tree(path: impl AsRef<Path>) -> impl Iterator<Item = (PathBuf, Metadata)> {
    walkdir::WalkDir::new(path).into_iter().filter_map(|entry| {
        let entry = entry.ok()?;

        if entry.file_type().is_file() {
            let meta = entry.metadata().ok()?;
            Some((PathBuf::from(entry.path()), meta))
        } else {
            None
        }
    })
}

fn imprint_mismatch(left: &Path, right: &Path) -> io::Result<bool> {
    Ok(Imprint::new(left)? == Imprint::new(right)?)
}
