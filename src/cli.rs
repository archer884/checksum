use std::{io::Write, path::Path};

use digest::Digest;

use crate::error::{Error, OperationKind};

/// check file hashes
///
/// Basic operation prints a file hash for a file. If two paths are given (both files or both
/// directories), the two will be compared. A comparison between directories makes use of the
/// Imprint type for greater efficiency.
///
/// If only a left-hand operand is provided, checksum will print the hash of the operand (assuing
/// said operand is a file; it is an error to pass in only a directory). The algorithm used for
/// this purpose may be set as an environment variable called CHECKSUM_DEF_ALG. Allowable names
/// include: blake3, md5, sha1, sha256, sha512. These are not case sensitive. This variable may
/// be set at compile time.
///
/// A further note on directory comparisons: directory comparisons are asymmetrical. Checksum
/// will ensure that all files from the left hand directory exist in the right hand directory
/// but not vice versa. This is for the common use case that files from the left have been copied
/// to some archive location on the right.
#[derive(Clone, Debug, clap::Parser)]
#[clap(about, version, author)]
pub struct Args {
    /// left hand resource
    pub left: String,

    /// right hand resource
    ///
    /// This resource, whether file or directory, is compared against the left. Both resources must
    /// be of matching type: e.g., if the left hand resource is a file, this must also be a file;
    /// if the left hand resource is a directory, this must also be a directory. This argument is
    /// ignored by all subcommands.
    pub right: Option<String>,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

impl Args {
    pub fn validate(&self) -> crate::Result<()> {
        // Validation is modal. If we've received a subcommand, we need to ensure that the left
        // hand path is a file. If we have not, we need to ensure that the category of the right
        // hand path matches the category of the left.

        let left = Path::new(&self.left);
        if self.command.is_some() {
            if !left.is_file() {
                return Err(Error::InvalidOperation(OperationKind::Hash));
            }
        } else if let Some(right) = &self.right {
            let right = Path::new(right);
            if left.is_file() && !right.is_file() || left.is_dir() && !right.is_dir() {
                return Err(Error::InvalidOperation(if left.is_file() {
                    OperationKind::File
                } else {
                    OperationKind::Dir
                }));
            }
        } else if !left.is_file() {
            return Err(Error::InvalidOperation(OperationKind::HashDir));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, clap::Subcommand)]
pub enum Command {
    Blake3(Blake3Command),
    Md5(Md5Command),
    Sha1(Sha1Command),
    Sha256(Sha256Command),
    Sha512(Sha512Command),
}

pub trait Mode {
    type Digest: Digest + Write;
    fn digest(&self) -> Self::Digest;
    fn get_hash(&self) -> Option<&str>;
}

macro_rules! impl_mode {
    ($command:ty, $digest:ty) => {
        impl Mode for $command {
            type Digest = $digest;

            fn digest(&self) -> Self::Digest {
                Default::default()
            }

            fn get_hash(&self) -> Option<&str> {
                self.hash.as_deref()
            }
        }
    };
}

/// blake3 mode
#[derive(Clone, Debug, clap::Parser)]
pub struct Blake3Command {
    /// blake3 hash
    ///
    /// Optional. If provided, checksum will assert that this hash matches the given file.
    hash: Option<String>,
}

impl_mode!(Blake3Command, blake3::Hasher);

/// md5 mode
#[derive(Clone, Debug, clap::Parser)]
pub struct Md5Command {
    /// md5 hash
    ///
    /// Optional. If provided, checksum will assert that this hash matches the given file.
    hash: Option<String>,
}

impl_mode!(Md5Command, md5::Md5);

/// sha1 mode
#[derive(Clone, Debug, clap::Parser)]
pub struct Sha1Command {
    /// sha1 hash
    ///
    /// Optional. If provided, checksum will assert that this hash matches the given file.
    hash: Option<String>,
}

impl_mode!(Sha1Command, sha1::Sha1);

/// sha256 mode
#[derive(Clone, Debug, clap::Parser)]
pub struct Sha256Command {
    /// sha256 hash
    ///
    /// Optional. If provided, checksum will assert that this hash matches the given file.
    hash: Option<String>,
}

impl_mode!(Sha256Command, sha2::Sha256);

/// sha512 mode
#[derive(Clone, Debug, clap::Parser)]
pub struct Sha512Command {
    /// sha512 hash
    ///
    /// Optional. If provided, checksum will assert that this hash matches the given file.
    hash: Option<String>,
}

impl_mode!(Sha512Command, sha2::Sha512);
