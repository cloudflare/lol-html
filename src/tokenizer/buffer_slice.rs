use std::convert::From;
use std::ops::Deref;
use std::fmt;
use std::str;

// NOTE: thin wrapper around byte slice that allows us pretty print tokens
#[derive(Default)]
pub struct BufferSlice<'t> {
    bytes: &'t [u8],
}

impl<'t> BufferSlice<'t> {
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.bytes) }
    }

    pub fn as_string(&self) -> String {
        unsafe { String::from_utf8_unchecked(self.bytes.to_vec()) }
    }
}

impl<'t> From<&'t [u8]> for BufferSlice<'t> {
    fn from(bytes: &'t [u8]) -> Self {
        BufferSlice { bytes }
    }
}

impl<'t> fmt::Debug for BufferSlice<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}

impl<'t> Deref for BufferSlice<'t> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.bytes
    }
}
