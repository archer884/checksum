mod alg;
mod cli;
mod error;
mod fmt;
mod hash;
mod hk;
mod iter;

use std::{
    io,
    path::{Path, PathBuf},
    process,
};

use alg::Algorithm;
use clap::Parser;
use cli::{Args, Command, Mode};
use error::OperationKind;
use hashbrown::HashMap;
use hk::Hashes;
use imprint::Imprint;
use iter::IsUniform;
use owo_colors::OwoColorize;
use rayon::prelude::*;
use uncased::{AsUncased, UncasedStr};

use crate::error::Error;

type Result<T, E = error::Error> = std::result::Result<T, E>;

fn main() {
    let args = Args::parse();

    if let Err(e) = run(&args) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    args.validate()?;

    // In the event we've received some subcommand, that's really the only thing we care about.
    // Each subcommand comes with a "mode" implementation that provides the right kind of hash
    // digest and access to an optional assertion, so all we have to pass in is the left path
    // and the mode. And hooray for static dispatch! This is going to generate the BEJEEZUS out of
    // some assembly, my friend.

    if let Some(command) = &args.command {
        return match command {
            Command::Blake3(mode) => execute_command(args, mode),
            Command::Md5(mode) => execute_command(args, mode),
            Command::Sha1(mode) => execute_command(args, mode),
            Command::Sha256(mode) => execute_command(args, mode),
            Command::Sha512(mode) => execute_command(args, mode),
        };
    }

    // If we haven't received any subcommands, check to see whether we've received a right-hand
    // resource. If so, we can safely assume (thanks to the validation call at the top) that both
    // resources are of the same type. (The same validation call ensured that the subcommand
    // comparisons were also valid.)

    if let Some(right) = &args.right {
        let left = Path::new(&args.left);

        return if left.is_file() {
            compare_files(args.left.as_ref(), right.as_ref())
        } else {
            compare_dirs(&args.left, right, args.full_comparison, args.verbose)
        };
    }

    // Last thing last: if we received no subcommand and no right hand-hand path, we just want to
    // print the hash of the left hand path. Exactly which algorithm we should use for this is
    // a matter of preference. Microsoft employs sha256 hashes for most checksums, whereas a lot
    // of content-addressed archives will name things using md5... I think what we're going to do
    // is to have the program ask whether we have a preference (read: check for an environment
    // variable) and, if not, fall back on md5 because it's short.

    // UNLESS the left-hand path is some kind of checksum file, in which case we want to use it
    // to verify any files.

    static CHECKSUM_FILE_EXTENSIONS: &[&str] = &[".md5", ".sha256"];

    let normalized_file_name = args.left.to_ascii_lowercase();
    if CHECKSUM_FILE_EXTENSIONS
        .iter()
        .copied()
        .any(|ext| normalized_file_name.ends_with(ext))
    {
        return apply_checksums(&args.left);
    }

    print_hash(&args.left)
}

fn print_hash(path: &str) -> Result<()> {
    let hash = if let Some(algorithm) = std::option_env!("CHECKSUM_DEF_ALG") {
        algorithm.parse::<Algorithm>()?.hash(path)?
    } else if let Ok(algorithm) = std::env::var("CHECKSUM_DEF_ALG") {
        algorithm.parse::<Algorithm>()?.hash(path)?
    } else {
        Algorithm::Md5.hash(path)?
    };

    println!("{hash}");

    Ok(())
}

fn apply_checksums(path: &str) -> Result<()> {
    let hashes = Hashes::from_path(path)?;

    for exception in hashes.verify() {
        let exception = exception?;
        println!("{exception}");
    }

    Ok(())
}

// FIXME: I want to adjust this so that it'll work with a directory or a list of files, but...
// I'm not real clear how I'm gonna make that happen.
//
// Hell, so far I'm not even writing a hash file.
fn execute_command(args: &Args, mode: &impl Mode) -> Result<()> {
    let left = hash::hash_to_string(&args.left, mode.digest())?;

    if let Some(right) = mode.get_hash() {
        return compare_hash_str(&left, right);
    }

    let (should_write, output) = mode.file_options();
    if should_write {
        // write hash file somehow...
        // I mean, the CLI works perfectly, but fuck me if my brain is interested in trying to
        // write this fuckin' file right now.
        dbg!(output);
    }

    println!("{left}");

    Ok(())
}

fn compare_hash_str(left: &str, right: &UncasedStr) -> Result<()> {
    if left.as_uncased() == right {
        let result = "True".green();
        println!("{result}");
        Ok(())
    } else {
        let result = "False".red();
        println!("{result}");
        process::exit(1);
    }
}

fn compare_files(left: &str, right: &str) -> Result<()> {
    let tasks = &[left, right];
    let tasks: io::Result<Vec<_>> = tasks
        .into_par_iter()
        .map(|&path| hash::hash_to_digest(path, blake3::Hasher::new()))
        .collect();

    if tasks?.uniform() {
        let result = "True".green();
        println!("{result}");
    } else {
        let result = "False".red();
        println!("{result}");
        process::exit(1);
    }

    Ok(())
}

trait Comparer {
    type Output: Eq;
    fn build(path: &Path) -> io::Result<Self::Output>;
}

#[derive(Clone, Copy)]
struct Blake3Comparer;

impl Comparer for Blake3Comparer {
    type Output = blake3::Hash;

    fn build(path: &Path) -> io::Result<Self::Output> {
        let mut hasher = blake3::Hasher::new();
        let mut reader = std::fs::File::open(path)?;
        io::copy(&mut reader, &mut hasher)?;
        Ok(hasher.finalize())
    }
}

#[derive(Clone, Copy)]
struct ImprintComparer;

impl Comparer for ImprintComparer {
    type Output = Imprint;

    fn build(path: &Path) -> io::Result<Self::Output> {
        Imprint::new(path)
    }
}

fn compare_with<T>(left: &Path, right: &Path) -> Result<bool>
where
    T: Comparer<Output: Send> + Copy,
{
    let tasks = &[left, right];
    let tasks: io::Result<Vec<_>> = tasks
        .into_par_iter()
        .map(move |&path| T::build(path))
        .collect();

    let uniform = tasks?.uniform();
    if !uniform {
        let mismatch = "MISMATCH".red();
        let path = left.display();
        println!("{mismatch} {path}");
    }

    Ok(uniform)
}

fn compare_dirs(left: &str, right: &str, full_comparison: bool, verbose: bool) -> Result<()> {
    ensure_distinct(left, right)?;

    let left = read_files(left).filter_map(|path| {
        get_relative_path(left.as_ref(), &path).map(|absolute| (absolute, path))
    });

    let right: HashMap<_, _> = read_files(right)
        .filter_map(|path| {
            get_relative_path(right.as_ref(), &path).map(|relative| (relative, path))
        })
        .collect();

    let has_failure = if full_comparison {
        compare_contents(left, &right, compare_with::<Blake3Comparer>, verbose)?
    } else {
        compare_contents(left, &right, compare_with::<ImprintComparer>, verbose)?
    };

    if !has_failure {
        let message = "True".green();
        println!("{message}");
    } else {
        process::exit(1);
    }

    Ok(())
}

fn ensure_distinct(left: &str, right: &str) -> Result<()> {
    let left = Path::new(left).canonicalize()?;
    let right = Path::new(right).canonicalize()?;

    if left.ancestors().any(|ancestor| ancestor == right)
        || right.ancestors().any(|ancestor| ancestor == left)
    {
        return Err(Error::InvalidOperation(OperationKind::Child));
    }

    Ok(())
}

fn compare_contents<I, C>(
    left: I,
    right: &HashMap<PathBuf, PathBuf>,
    compare: C,
    verbose: bool,
) -> Result<bool>
where
    I: IntoIterator<Item = (PathBuf, PathBuf)>,
    C: Fn(&Path, &Path) -> Result<bool>,
{
    let message = "match".green();
    let mut has_failure = false;
    for (relative, absolute) in left {
        if let Some(right_hand_absolute_path) = right.get(&relative) {
            if !compare(&absolute, right_hand_absolute_path)? {
                has_failure = true;
            } else if verbose {
                let path = relative.display();
                println!("{message} {path}");
            }
        } else {
            let missing = "missing".yellow();
            let relative = relative.display();
            println!("{missing} {relative}");
            has_failure = true;
        }
    }
    Ok(has_failure)
}

fn read_files(path: &str) -> impl Iterator<Item = PathBuf> {
    // let files = fs::read_dir(path)?.filter_map(|entry| {
    let files = walkdir::WalkDir::new(path).into_iter().filter_map(|entry| {
        let entry = entry.ok()?;
        let meta = entry.metadata().ok()?;

        if meta.file_type().is_file() {
            Some(entry.into_path())
        } else {
            None
        }
    });

    files.filter(|path| {
        path.file_name()
            .map(|name| !name.to_string_lossy().starts_with('.'))
            .unwrap_or_default()
    })
}

fn get_relative_path(base: &Path, path: &Path) -> Option<PathBuf> {
    path.strip_prefix(base).map(|path| path.to_owned()).ok()
}
