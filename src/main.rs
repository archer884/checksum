use std::path::Path;
use std::{fs, io};
use sha2::{Digest, Sha256};
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
struct Opt {
    /// The path of the file to be hashed.
    path: String,
    /// A path to compare against.
    #[structopt(long = "eq")]
    other_path: Option<String>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    match opt.other_path {
        Some(other_path) => compare(opt.path, other_path)?,
        None => display_hash(opt.path)?,
    }

    Ok(())
}

fn compare(left: impl AsRef<Path>, right: impl AsRef<Path>) -> io::Result<()> {
    if hash(left)? == hash(right)? {
        println!("True");
    } else {
        println!("False");
    }
    Ok(())
}

fn display_hash(path: impl AsRef<Path>) -> io::Result<()> {
    use std::fmt::{self, LowerHex};
    
    struct LowerHexFormatter(Vec<u8>);

    impl LowerHex for LowerHexFormatter {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for u in self.0.iter() {
                write!(f, "{:x}", u)?;
            }
            Ok(())
        }
    }
    
    println!("{:x}", LowerHexFormatter(hash(path)?));
    Ok(())
}

fn hash(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut hasher = Sha256::new();
    let mut reader = fs::File::open(path)?;

    io::copy(&mut reader, &mut hasher)?;

    Ok(Vec::from(hasher.result().as_slice()))
}
