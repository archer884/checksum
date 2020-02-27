use std::fmt::{self, LowerHex};

pub struct LowerHexFormatter(pub Vec<u8>);

impl LowerHex for LowerHexFormatter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for u in &self.0 {
            u.fmt(f)?;
        }
        Ok(())
    }
}

// This is neat, but not useful.
// impl<T> LowerHex for LowerHexFormatter<T>
// where
//     for<'a> &'a T: IntoIterator,
//     for<'a> <&'a T as IntoIterator>::Item: LowerHex,
// {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         for u in &self.0 {
//             u.fmt(f)?;
//         }
//         Ok(())
//     }
// }
