use std::fmt::{self, LowerHex};

pub struct LowerHexFormatter(pub Vec<u8>);

impl LowerHex for LowerHexFormatter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for u in &self.0 {
            write!(f, "{:02x}", u)?;
        }
        Ok(())
    }
}
