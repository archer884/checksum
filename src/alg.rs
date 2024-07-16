use core::fmt;
use std::{io, path::Path, str::FromStr};

use crate::error::Error;

#[derive(Clone, Copy, Debug, Default)]
pub enum Algorithm {
    Blake3,
    Md5,
    #[default]
    Sha1,
    Sha256,
    Sha512,
}

impl Algorithm {
    #[inline]
    pub fn hash(self, path: impl AsRef<Path>) -> io::Result<String> {
        match self {
            Algorithm::Blake3 => crate::hash::hash_to_string(path, blake3::Hasher::new()),
            Algorithm::Md5 => crate::hash::hash_to_string(path, md5::Md5::default()),
            Algorithm::Sha1 => crate::hash::hash_to_string(path, sha1::Sha1::default()),
            Algorithm::Sha256 => crate::hash::hash_to_string(path, sha2::Sha256::default()),
            Algorithm::Sha512 => crate::hash::hash_to_string(path, sha2::Sha512::default()),
        }
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Algorithm::Blake3 => f.write_str("Blake3"),
            Algorithm::Md5 => f.write_str("Md5"),
            Algorithm::Sha1 => f.write_str("Sha1"),
            Algorithm::Sha256 => f.write_str("Sha256"),
            Algorithm::Sha512 => f.write_str("Sha512"),
        }
    }
}

impl FromStr for Algorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_ref() {
            "BLAKE3" => Ok(Algorithm::Blake3),
            "MD5" => Ok(Algorithm::Md5),
            "SHA1" => Ok(Algorithm::Sha1),
            "SHA256" => Ok(Algorithm::Sha256),
            "SHA512" => Ok(Algorithm::Sha512),
            _ => Err(Error::UnknownAlgorithm(s.into())),
        }
    }
}
