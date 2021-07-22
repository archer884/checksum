use std::{
    fs,
    fs::Metadata,
    io::{self, Write},
    path::{Path, PathBuf},
    process,
    str::FromStr,
};

mod fmt;
mod iter;

use fmt::LowerHexFormatter;
use hashbrown::HashMap;
use imprint::Imprint;
use md5::Md5;
use sha1::{Digest, Sha1};
use sha2::Sha256;

enum Algorithm {
    Md5,
    Sha1,
    Sha256,
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm::Sha1
    }
}

impl FromStr for Algorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let algorithm = s.to_ascii_lowercase();
        match &*algorithm {
            "md5" => Ok(Algorithm::Md5),
            "sha" | "sha2" | "sha256" => Ok(Algorithm::Sha256),
            _ => Err("use sha256 or md5"),
        }
    }
}

struct CompareTrees {
    pub left: String,
    pub right: String,
    pub force: bool,
    pub include_hidden_files: bool,
}

enum Command {
    Print {
        path: String,
    },
    Assert {
        path: String,
        checksum: String,
        algorithm: Option<Algorithm>,
    },
    Compare {
        left: String,
        right: String,
    },
    CompareTrees(CompareTrees),
}

fn read_command() -> Command {
    use clap::{App, AppSettings, Arg};

    let assert = App::new("assert")
        .about("Assert that a file matches a given checksum")
        .args(&[
            Arg::new("path")
                .about("a file path")
                .required(true)
                .takes_value(true),
            Arg::new("checksum")
                .about("a file path")
                .required(true)
                .takes_value(true),
            Arg::new("algorithm")
                .about("a hash algorithm")
                .short('a')
                .long("algorithm")
                .required(false)
                .takes_value(true),
        ]);

    let compare = App::new("compare").about("Compare two files").args(&[
        Arg::new("left")
            .about("a file path")
            .required(true)
            .takes_value(true),
        Arg::new("right")
            .about("a second file path")
            .required(true)
            .takes_value(true),
    ]);

    let compare_trees = App::new("compare-trees")
        .about("Compare two directory trees")
        .args(&[
            Arg::new("left")
                .about("a directory path")
                .required(true)
                .takes_value(true),
            Arg::new("right")
                .about("a second directory path")
                .required(true)
                .takes_value(true),
            Arg::new("force")
                .short('f')
                .long("force")
                .about("test all paths")
                .long_about("do not abort tree comparison at the first mismatch")
                .required(false)
                .takes_value(false),
            Arg::new("hidden")
                .short('h')
                .long("hidden")
                .about("compare hidden files")
                .required(false)
                .takes_value(false),
        ]);

    let args = App::new("checksum")
        .global_setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            Arg::new("path")
                .about("a file path")
                .required(true)
                .takes_value(true),
        )
        .subcommand(assert)
        .subcommand(compare)
        .subcommand(compare_trees)
        .get_matches();

    if let Some(sub) = args.subcommand_matches("assert") {
        return Command::Assert {
            path: sub.value_of_t_or_exit("path"),
            checksum: sub.value_of_t_or_exit("checksum"),
            algorithm: if sub.is_present("algorithm") {
                Some(sub.value_of_t_or_exit("algorithm"))
            } else {
                None
            },
        };
    }

    if let Some(sub) = args.subcommand_matches("compare") {
        return Command::Compare {
            left: sub.value_of_t_or_exit("left"),
            right: sub.value_of_t_or_exit("right"),
        };
    }

    if let Some(sub) = args.subcommand_matches("compare-trees") {
        return Command::CompareTrees(CompareTrees {
            left: sub.value_of("left").unwrap().to_string(),
            right: sub.value_of("right").unwrap().to_string(),
            force: sub.is_present("force"),
            include_hidden_files: sub.is_present("hidden"),
        });
    }

    Command::Print {
        path: args.value_of("path").unwrap().to_string(),
    }
}

fn main() -> io::Result<()> {
    match read_command() {
        Command::Print { path } => display_hash(path),
        Command::Assert {
            path,
            checksum,
            algorithm,
        } => assert(path, checksum, algorithm.unwrap_or_default()),
        Command::Compare { left, right } => compare(left, right),
        Command::CompareTrees(compare) => compare_trees(&compare),
    }
}

fn assert(path: impl AsRef<Path>, expected: String, algorithm: Algorithm) -> io::Result<()> {
    let hash = match algorithm {
        Algorithm::Sha1 => hash_sha1(path)?,
        Algorithm::Sha256 => hash_sha256(path)?,
        Algorithm::Md5 => hash_md5(path)?,
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
    let tasks: io::Result<Vec<_>> = [left, right].into_par_iter().map(hash_sha256).collect();

    if tasks?.uniform() {
        println!("True");
    } else {
        println!("False");
        process::exit(1);
    }

    Ok(())
}

// fn compare_trees<T: AsRef<Path> + Send>(left: T, right: T, force: bool) -> io::Result<()> {
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

fn display_hash(path: impl AsRef<Path>) -> io::Result<()> {
    println!("{:x}", LowerHexFormatter(hash_sha256(path)?));
    Ok(())
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
