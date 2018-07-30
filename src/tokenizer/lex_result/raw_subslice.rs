use super::shallow_token::SliceRange;
use std::convert::From;
use std::fmt;
use std::ops::Deref;
use std::str;

// NOTE: a thin wrapper around token's raw bytes subslice that allows us pretty print tokens
#[derive(Default)]
pub struct RawSubslice<'t> {
    bytes: &'t [u8],
}

impl<'t> RawSubslice<'t> {
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.bytes) }
    }

    pub fn as_string(&self) -> String {
        unsafe { String::from_utf8_unchecked(self.bytes.to_vec()) }
    }
}

impl<'t> From<&'t [u8]> for RawSubslice<'t> {
    fn from(bytes: &'t [u8]) -> Self {
        RawSubslice { bytes }
    }
}

impl<'t> From<(&'t [u8], SliceRange)> for RawSubslice<'t> {
    fn from((raw, range): (&'t [u8], SliceRange)) -> Self {
        (&raw[range.start..range.end]).into()
    }
}

impl<'t> fmt::Debug for RawSubslice<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}

impl<'t> Deref for RawSubslice<'t> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.bytes
    }
}
