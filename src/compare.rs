use std::{
    io::{self, IsTerminal},
    path::{Path, PathBuf},
};

use hashbrown::HashMap;
use imprint::Imprint;
use owo_colors::OwoColorize;
use rayon::prelude::*;

use crate::iter::IsUniform;

pub trait Comparer {
    type Output: Eq;
    fn build(path: &Path) -> io::Result<Self::Output>;
}

#[derive(Clone, Copy)]
pub struct Blake3Comparer;

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
pub struct ImprintComparer;

impl Comparer for ImprintComparer {
    type Output = Imprint;

    fn build(path: &Path) -> io::Result<Self::Output> {
        Imprint::new(path)
    }
}

pub fn compare_contents<C>(
    left: impl IntoIterator<Item = (PathBuf, PathBuf)>,
    right: &HashMap<PathBuf, PathBuf>,
    verbose: bool,
) -> crate::Result<bool>
where
    C: Comparer<Output: Send> + Copy,
{
    let colorize = io::stdout().is_terminal();
    
    let message = "match".green();
    let mut has_failure = false;

    for (relative, absolute) in left {
        if let Some(right_hand_absolute_path) = right.get(&relative) {
            if !compare_with::<C>(&absolute, right_hand_absolute_path, colorize)? {
                has_failure = true;
            } else if verbose {
                let path = relative.display();
                println!("{message} {path}");
            }
        } else {
            print_missing(relative, colorize);
            has_failure = true;
        }
    }
    
    Ok(has_failure)
}

fn print_missing(relative: PathBuf, colorize: bool) {
    if colorize {
        let missing = "missing".yellow();
        let relative = relative.display();
        println!("{missing} {relative}");
    } else {
        let relative = relative.display();
        println!("missing {relative}");
    }
}

pub fn compare_with<T>(left: &Path, right: &Path, colorize: bool) -> crate::Result<bool>
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
        if colorize {
            let mismatch = "MISMATCH".red();
            let path = left.display();
            println!("{mismatch} {path}");
        } else {
            println!("MISMATCH {}", left.display());
        }
    }

    Ok(uniform)
}
