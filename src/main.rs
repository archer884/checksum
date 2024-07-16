mod alg;
mod cli;
mod compare;
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

use cli::{Args, Command, FileCommand};
use compare::{Blake3Comparer, ImprintComparer};
use error::OperationKind;
use hashbrown::HashMap;
use hk::Hashes;
use iter::IsUniform;
use owo_colors::OwoColorize;
use rayon::prelude::*;
use uncased::AsUncased;

use crate::error::Error;

/// Environment key defining the default algorithm for this program.
static CHECKSUM_DEFAULT_ALG: &str = "CHECKSUM_DEFAULT_ALG";

type Result<T, E = error::Error> = std::result::Result<T, E>;

fn main() {
    if let Err(e) = run(&Args::parse()) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    args.validate()?;

    // First thing first, the primary arguments do not apply in the event we've received a
    // subcommand. In that case, we'll handle the subcommand and return.

    if let Some(command) = &args.command {
        return dispatch_command(args, command);
    }

    // We've got a set of arguments, but at this juncture we don't know what those arguments
    // represent. The first thing we need to do is to establish what these things actually are.
    // What we know so far is that, if the left hand argument references a directory, so does
    // the right hand argument. However, in the event that the left hand argument references a
    // file, the right hand argument might be either a file or a checksum value.

    if let Some(compare) = args.compare.as_deref() {
        let target: &Path = args.target().as_ref();
        if target.is_file() {
            return compare_files(args.target(), compare);
        } else {
            return compare_dirs(args.target(), compare, DirCompareContext(args));
        }
    }

    // If instead the user has requested a hash assertion, we'll compare the hashed version of
    // the file against the asserted hash. It is the user's responsibility to select the correct
    // comparison mode, so we'll just hope he's done that.

    if let Some(hash) = args.assert.as_deref() {
        let target = args.mode().hash(args.target())?;
        return compare_hash_str(&target, hash);
    }

    // If we have come this far, it's because the user has not selected either a file, directory,
    // or hash comparison. In that case, our job has finally become very simple.
    print_hash(args)
}

fn print_hash(args: &Args) -> Result<()> {
    let path = args.target();
    let mode = args.mode();

    let files: Vec<_> = read_files(path).collect();
    for file in &files {
        if files.len() == 1 {
            println!("{}", mode.hash(file)?);
        } else {
            println!("{}  {}", mode.hash(file)?, file.display());
        }
    }

    Ok(())
}

fn dispatch_command(_args: &Args, command: &Command) -> Result<()> {
    match command {
        Command::File(FileCommand { path }) => apply_checksums(path),
    }
}

fn apply_checksums(path: &str) -> Result<()> {
    let hashes = Hashes::from_path(path)?;

    for exception in hashes.verify() {
        let exception = exception?;
        println!("{exception}");
    }

    Ok(())
}

fn compare_hash_str(left: &str, right: &str) -> Result<()> {
    let right = right.as_uncased();
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

pub struct DirCompareContext<'a>(&'a Args);

impl DirCompareContext<'_> {
    #[inline]
    fn full_comparison(&self) -> bool {
        self.0.force_full_compare
    }

    #[inline]
    fn verbose(&self) -> bool {
        self.0.verbose
    }
}

fn compare_dirs(left: &str, right: &str, context: DirCompareContext) -> Result<()> {
    ensure_distinct(left, right)?;

    let left = read_files(left).filter_map(|path| {
        get_relative_path(left.as_ref(), &path).map(|absolute| (absolute, path))
    });

    let right: HashMap<_, _> = read_files(right)
        .filter_map(|path| {
            get_relative_path(right.as_ref(), &path).map(|relative| (relative, path))
        })
        .collect();

    let has_failure = if context.full_comparison() {
        compare::compare_contents::<Blake3Comparer>(left, &right, context.verbose())?
    } else {
        compare::compare_contents::<ImprintComparer>(left, &right, context.verbose())?
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

fn read_files(path: &str) -> impl Iterator<Item = PathBuf> {
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
            .map_or(false, |name| !name.to_string_lossy().starts_with('.'))
    })
}

fn get_relative_path(base: &Path, path: &Path) -> Option<PathBuf> {
    path.strip_prefix(base).map(|path| path.to_owned()).ok()
}
