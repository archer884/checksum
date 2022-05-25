use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

use digest::{Digest, Output};

pub fn hash_to_digest<T: Digest + Write>(
    path: impl AsRef<Path>,
    mut digest: T,
) -> io::Result<Output<T>> {
    let mut reader = File::open(path)?;
    io::copy(&mut reader, &mut digest)?;
    Ok(digest.finalize())
}

pub fn hash_to_string<T: Digest + Write>(path: impl AsRef<Path>, digest: T) -> io::Result<String> {
    hash_to_digest(path, digest).map(|result| fmt_hex(result.as_slice()))
}

fn fmt_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut buf = String::with_capacity(bytes.len() * 2);
    for &u in bytes {
        write!(buf, "{u:02x}").unwrap();
    }
    buf
}
