use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
};

mod cli;
mod error;
mod fmt;
mod hash;
mod iter;

use clap::Parser;
use cli::{Args, Command, Mode};
use hashbrown::HashMap;
use imprint::Imprint;
use iter::IsUniform;
use owo_colors::OwoColorize;
use rayon::prelude::*;

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
            Command::Blake3(mode) => execute_command(&args.left, mode),
            Command::Md5(mode) => execute_command(&args.left, mode),
            Command::Sha1(mode) => execute_command(&args.left, mode),
            Command::Sha256(mode) => execute_command(&args.left, mode),
            Command::Sha512(mode) => execute_command(&args.left, mode),
        };
    }

    // If we haven't received any subcommands, check to see whether we've received a right-hand
    // resource. If so, we can safely assume (thanks to the validation call at the top) that both
    // resources are of the same type. (The same validation call ensured that the subcommand
    // comparisons were also valid.)

    if let Some(right) = &args.right {
        let left = Path::new(&args.left);

        return if left.is_file() {
            compare_files(&args.left, right)
        } else {
            compare_dirs(&args.left, right)
        };
    }

    // Last thing last: if we received no subcommand and no right hand-hand path, we just want to
    // print the hash of the left hand path. Exactly which algorithm we should use for this is
    // a matter of preference. Microsoft employs sha256 hashes for most checksums, whereas a lot
    // of content-addressed archives will name things using md5... I think what we're going to do
    // is to have the program ask whether we have a preference (read: check for an environment
    // variable) and, if not, fall back on md5 because it's short.

    #[inline]
    fn hash_by_algorithm_name(path: &str, name: &str) -> Result<String> {
        let hash = match name.to_ascii_uppercase().as_ref() {
            "BLAKE3" => hash::hash_to_string(path, blake3::Hasher::new()),
            "MD5" => hash::hash_to_string(path, md5::Md5::default()),
            "SHA1" => hash::hash_to_string(path, sha1::Sha1::default()),
            "SHA256" => hash::hash_to_string(path, sha2::Sha256::default()),
            "SHA512" => hash::hash_to_string(path, sha2::Sha512::default()),
            _ => return Err(Error::UnknownAlgorithm(name.into())),
        };
        Ok(hash?)
    }

    let hash = if let Some(algorithm) = std::option_env!("CHECKSUM_DEF_ALG") {
        hash_by_algorithm_name(&args.left, algorithm)?
    } else if let Ok(algorithm) = std::env::var("CHECKSUM_DEF_ALG") {
        hash_by_algorithm_name(&args.left, &algorithm)?
    } else {
        hash::hash_to_string(&args.left, md5::Md5::default())?
    };

    println!("{hash}");

    Ok(())
}

fn execute_command(path: impl AsRef<Path>, mode: &impl Mode) -> Result<()> {
    let path = path.as_ref();
    let left = hash::hash_to_string(path, mode.digest())?;

    if let Some(right) = mode.get_hash() {
        if left == right {
            let result = "True".green();
            println!("{result}");
        } else {
            let result = "False".red();
            println!("{result}");
            process::exit(1);
        }
    } else {
        println!("{left}");
    }

    Ok(())
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

fn short_compare_files(left: &Path, right: &Path) -> Result<bool> {
    let tasks = &[left, right];
    let tasks: io::Result<Vec<_>> = tasks.into_par_iter().map(Imprint::new).collect();

    let uniform = tasks?.uniform();
    if !uniform {
        let mismatch = "MISMATCH".red();
        let path = left.display();
        println!("{mismatch} {path}");
    }

    Ok(uniform)
}

fn compare_dirs(left: &str, right: &str) -> Result<()> {
    let left = read_files(left)?.filter_map(|path| {
        get_relative_path(left.as_ref(), &path).map(|absolute| (absolute, path))
    });

    let right: HashMap<_, _> = read_files(right)?
        .filter_map(|path| {
            get_relative_path(right.as_ref(), &path).map(|relative| (relative, path))
        })
        .collect();

    let mut has_failure = false;
    for (relative, absolute) in left {
        if let Some(right_hand_absolute_path) = right.get(&relative) {
            if !short_compare_files(&absolute, right_hand_absolute_path)? {
                has_failure = true;
            }
        } else {
            let missing = "missing".yellow();
            let relative = relative.display();
            println!("{missing} {relative}");
            has_failure = true;
        }
    }

    if !has_failure {
        let message = "True".green();
        println!("{message}");
    } else {
        process::exit(1);
    }

    Ok(())
}

fn read_files(path: &str) -> io::Result<impl Iterator<Item = PathBuf>> {
    let files = fs::read_dir(path)?.filter_map(|entry| {
        let entry = entry.ok()?;
        let meta = entry.metadata().ok()?;

        if meta.file_type().is_file() {
            Some(entry.path())
        } else {
            None
        }
    });

    let non_hidden_files = files.filter(|path| {
        path.file_name()
            .map(|name| !name.to_string_lossy().starts_with('.'))
            .unwrap_or_default()
    });

    Ok(non_hidden_files)
}

fn get_relative_path(base: &Path, path: &Path) -> Option<PathBuf> {
    path.strip_prefix(base).map(|path| path.to_owned()).ok()
}
