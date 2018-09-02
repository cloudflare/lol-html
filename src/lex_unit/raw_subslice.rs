use super::shallow_token::SliceRange;
use std::convert::From;
use std::ops::Deref;

#[cfg(feature = "testing_api")]
use std::{fmt, str};

// NOTE: a thin wrapper around token's raw bytes subslice that allows us pretty print tokens
#[derive(Default)]
pub struct RawSubslice<'t>(&'t [u8]);

#[cfg(feature = "testing_api")]
impl<'t> RawSubslice<'t> {
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self) }
    }

    pub fn as_string(&self) -> String {
        unsafe { String::from_utf8_unchecked(self.to_vec()) }
    }
}

impl<'t> From<&'t [u8]> for RawSubslice<'t> {
    fn from(bytes: &'t [u8]) -> Self {
        RawSubslice(bytes)
    }
}

impl<'t> From<(&'t [u8], SliceRange)> for RawSubslice<'t> {
    fn from((raw, range): (&'t [u8], SliceRange)) -> Self {
        (&raw[range.start..range.end]).into()
    }
}

#[cfg(feature = "testing_api")]
impl<'t> fmt::Debug for RawSubslice<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}

impl<'t> Deref for RawSubslice<'t> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0
    }
}
