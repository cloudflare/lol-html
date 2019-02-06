use encoding_rs::{Encoding, WINDOWS_1252};
use memchr::{memchr, memchr3};
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

    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> Bytes<'static> {
        Bytes(Cow::Owned(self.to_vec()))
    }

    #[inline]
    pub fn opt_to_owned(bytes: &Option<Bytes<'_>>) -> Option<Bytes<'static>> {
        bytes.as_ref().map(|b| b.to_owned())
    }

    pub(crate) fn as_debug_string(&self) -> String {
        // NOTE: use WINDOWS_1252 (superset of ASCII) encoding here as
        // the most safe variant since we don't know which actual encoding
        // has been used for bytes.
        self.as_string(WINDOWS_1252)
    }
}

macro_rules! impl_replace_byte {
    ($self:tt, $output_handler:ident, $impls:ident) => {
        let mut tail: &[u8] = $self;

        loop {
            match $impls!(@find tail) {
                Some(pos) => {
                    let replacement = $impls!(@get_replacement tail, pos);

                    $output_handler(&tail[..pos]);
                    $output_handler(&replacement);
                    tail = &tail[pos + 1..];
                }
                None => {
                    if !tail.is_empty() {
                        $output_handler(&tail);
                    }
                    break;
                }
            }
        }
    };
}

impl<'b> Bytes<'b> {
    #[inline]
    pub fn replace_byte(&self, (needle, repl): (u8, &[u8]), output_handler: &mut dyn FnMut(&[u8])) {
        macro_rules! impls {
            (@find $tail:ident) => {
                memchr(needle, $tail)
            };

            (@get_replacement $tail:ident, $pos:ident) => {
                repl
            };
        }

        impl_replace_byte!(self, output_handler, impls);
    }

    #[inline]
    pub fn replace_byte3(
        &self,
        (needle1, repl1): (u8, &[u8]),
        (needle2, repl2): (u8, &[u8]),
        (needle3, repl3): (u8, &[u8]),
        output_handler: &mut dyn FnMut(&[u8]),
    ) {
        macro_rules! impls {
            (@find $tail:ident) => {
                memchr3(needle1, needle2, needle3, $tail)
            };

            (@get_replacement $tail:ident, $pos:ident) => {{
                let matched = $tail[$pos];

                if matched == needle1 {
                    repl1
                } else if matched == needle2 {
                    repl2
                } else {
                    repl3
                }
            }};
        }

        impl_replace_byte!(self, output_handler, impls);
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
