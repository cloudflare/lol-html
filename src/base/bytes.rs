use std::fmt::{self, Debug};
use std::ops::Deref;
use std::str;

/// Bytes is a thin wrapper around a byte slice with some handy APIs
#[repr(transparent)]
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

impl<'b> Debug for Bytes<'b> {
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
