use base::Chunk;
use std::ops::Deref;
use std::{fmt, str};

/// Bytes is a thin wrapper around a byte slice with some handy APIs
pub struct Bytes<'b>(&'b [u8]);

impl<'b> Bytes<'b> {
    pub fn as_str(&self) -> &str {
        str::from_utf8(self).unwrap()
    }

    pub fn as_string(&self) -> String {
        String::from_utf8(self.to_vec()).unwrap()
    }
}

impl<'b> From<&'b [u8]> for Bytes<'b> {
    fn from(bytes: &'b [u8]) -> Self {
        Bytes(bytes)
    }
}

impl<'b> From<&'b Chunk<'b>> for Bytes<'b> {
    fn from(chunk: &'b Chunk<'b>) -> Self {
        chunk.into()
    }
}

impl<'b> fmt::Debug for Bytes<'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}

impl<'b> Deref for Bytes<'b> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0
    }
}
