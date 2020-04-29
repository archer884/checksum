mod fmt;
mod iter;

use fmt::LowerHexFormatter;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::{fs, io, process};

enum Command {
    Print { path: String },
    Assert { path: String, checksum: String },
    Compare { left: String, right: String },
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

    Command::Print {
        path: args.value_of("path").unwrap().to_string(),
    }
}

fn main() -> io::Result<()> {
    match read_command() {
        Command::Print { path } => display_hash(path),
        Command::Assert { path, checksum } => assert(path, checksum),
        Command::Compare { left, right } => compare(left, right),
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
