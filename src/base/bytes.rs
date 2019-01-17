use encoding_rs::{Encoding, WINDOWS_1252};
use memchr::memchr;
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::str;

/// Bytes is a thin wrapper around either byte slice or
/// owned bytes with some handy APIs attached
#[repr(transparent)]
pub struct Bytes<'b>(Cow<'b, [u8]>);

impl<'b> Bytes<'b> {
    #[inline]
    pub fn empty() -> Self {
        b"".into()
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
    pub fn replace_ch(&self, ch: u8, replacement: &[u8], chunk_handler: &mut dyn FnMut(Bytes<'_>)) {
        let mut remainder: &[u8] = self;

        loop {
            let pos = memchr(ch, remainder);

            match pos {
                Some(pos) => {
                    chunk_handler(remainder[..pos].into());
                    chunk_handler(replacement.into());
                    remainder = &remainder[pos + 1..];
                }
                None => {
                    if !remainder.is_empty() {
                        chunk_handler(remainder.into());
                    }
                    break;
                }
            }
        }
    }

    #[inline]
    pub fn into_owned(self) -> Bytes<'static> {
        Bytes(Cow::Owned(self.0.into_owned()))
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

macro_rules! impl_from_static {
    ($($size:expr),+) => {
        $(
            impl<'b> From<&'b [u8; $size]> for Bytes<'b> {
                #[inline]
                fn from(bytes: &'b [u8; $size]) -> Self {
                    Bytes(Cow::Borrowed(bytes))
                }
            }
        )+
    };
}

impl_from_static!(0, 1, 2);

impl Clone for Bytes<'_> {
    // NOTE: usually bytes are bound to the lifetime of the current input chunk.
    // To unbound tokens from the original input after Clone `clone` implementation
    // for bytes creates deep copy of the content (unlike Cow, which preserves references).
    #[inline]
    fn clone(&self) -> Bytes<'static> {
        Bytes(Cow::Owned(self.to_vec()))
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
