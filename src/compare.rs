use std::{
    io,
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
    let message = "match".green();
    let mut has_failure = false;
    for (relative, absolute) in left {
        if let Some(right_hand_absolute_path) = right.get(&relative) {
            if !compare_with::<C>(&absolute, right_hand_absolute_path)? {
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

pub fn compare_with<T>(left: &Path, right: &Path) -> crate::Result<bool>
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
