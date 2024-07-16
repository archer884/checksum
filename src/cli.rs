use std::path::Path;

use clap::{ArgGroup, Parser, Subcommand};

use crate::{
    alg::Algorithm,
    error::{Error, OperationKind},
    CHECKSUM_DEFAULT_ALG,
};

#[derive(Clone, Debug, Parser)]
#[command(subcommand_negates_reqs(true), group(ArgGroup::new("compare to")))]
pub struct Args {
    /// a file or directory
    #[arg(required = true)]
    pub target: Option<String>,

    /// a file or directory
    ///
    /// Provide this argument to assert that the target and comparison targets
    /// are equal. Non-equal files will be printed to the screen. Any non-equal
    /// files will cause the command to return an error code to the shell.
    #[arg(short, long, group("compare to"))]
    pub compare: Option<String>,

    /// a hash value
    ///
    /// Provide this argument to assert that the target and hash are equal.
    #[arg(short, long, group("compare to"))]
    pub assert: Option<String>,

    /// the hashing algorithm to be used
    ///
    /// For output, the default is sha256, but the default algorithm may be overridden
    /// by setting an environment variable called CHECKSUM_DEFAULT_ALG.
    ///
    /// For internal comparisons, checksum uses Blake3.
    #[arg(short, long, env(CHECKSUM_DEFAULT_ALG))]
    mode: Option<Algorithm>,

    /// force full comparison
    ///
    /// Comparisons between directory trees are partial comparisons
    /// by default. Pass this flag to trigger a full comparison. A
    /// full comparison is MUCH SLOWER.
    #[arg(short, long)]
    pub force_full_compare: bool,

    /// print names of matching files during directory comparisons
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }

    pub fn target(&self) -> &str {
        self.target
            .as_deref()
            .expect("DO NOT CALL THIS METHOD IF A SUBCOMMAND IS PASSED")
    }

    pub fn mode(&self) -> Algorithm {
        self.mode.unwrap_or_default()
    }

    pub fn validate(&self) -> crate::Result<()> {
        let Some(target) = &self.target else {
            return Ok(());
        };

        let Some(compare) = &self.compare else {
            return Ok(());
        };

        let left: &Path = target.as_ref();
        let right: &Path = compare.as_ref();

        if left.is_dir() && !right.is_dir() {
            return Err(Error::InvalidOperation(OperationKind::Dir));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    File(FileCommand),
}

#[derive(Clone, Debug, Parser)]
pub struct FileCommand {
    pub path: String,
}
