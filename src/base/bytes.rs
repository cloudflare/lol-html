use encoding_rs::{Encoding, WINDOWS_1252};
use memchr::memchr;
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::str;

// Bytes is a thin wrapper around either byte slice or
// owned bytes with some handy APIs attached
#[repr(transparent)]
pub struct Bytes<'b>(Cow<'b, [u8]>);

impl<'b> Bytes<'b> {
    #[inline]
    pub fn empty() -> Self {
        Bytes(Cow::Borrowed(&[]))
    }

    #[inline]
    pub fn from_str(string: &'b str, encoding: &'static Encoding) -> Self {
        encoding.encode(string).0.into()
    }

    #[inline]
    pub fn from_str_without_replacements(
        string: &'b str,
        encoding: &'static Encoding,
    ) -> Option<Self> {
        let (res, _, has_replacements) = encoding.encode(string);

        if has_replacements {
            None
        } else {
            Some(res.into())
        }
    }

    #[inline]
    pub fn as_string(&self, encoding: &'static Encoding) -> String {
        encoding.decode(self).0.into_owned()
    }

    #[inline]
    pub fn into_owned(self) -> Bytes<'static> {
        Bytes(Cow::Owned(self.0.into_owned()))
    }

    #[inline]
    pub fn replace_byte(&self, (needle, repl): (u8, &[u8]), output_handler: &mut dyn FnMut(&[u8])) {
        let mut tail: &[u8] = self;

        loop {
            match memchr(needle, tail) {
                Some(pos) => {
                    output_handler(&tail[..pos]);
                    output_handler(&repl);
                    tail = &tail[pos + 1..];
                }
                None => {
                    if !tail.is_empty() {
                        output_handler(&tail);
                    }
                    break;
                }
            }
        }
    }

    pub(crate) fn as_debug_string(&self) -> String {
        // NOTE: use WINDOWS_1252 (superset of ASCII) encoding here as
        // the most safe variant since we don't know which actual encoding
        // has been used for bytes.
        self.as_string(WINDOWS_1252)
    }
}

impl<'b> From<Cow<'b, [u8]>> for Bytes<'b> {
    #[inline]
    fn from(bytes: Cow<'b, [u8]>) -> Self {
        Bytes(bytes)
    }
}

impl<'b> From<&'b [u8]> for Bytes<'b> {
    #[inline]
    fn from(bytes: &'b [u8]) -> Self {
        Bytes(bytes.into())
    }
}

impl Debug for Bytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.as_debug_string())
    }
}

impl Deref for Bytes<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &*self.0
    }
}
