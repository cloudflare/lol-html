use encoding_rs::{Encoding, WINDOWS_1252};
use memchr::{memchr, memchr2};
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
    ($self:tt, $chunk_handler:ident, $impls:ident) => {
        let mut tail: &[u8] = $self;

        loop {
            match $impls!(@find tail) {
                Some(pos) => {
                    let replacement = $impls!(@get_replacement tail, pos);

                    $chunk_handler(&tail[..pos].into());
                    $chunk_handler(&replacement.into());
                    tail = &tail[pos + 1..];
                }
                None => {
                    if !tail.is_empty() {
                        $chunk_handler(&tail.into());
                    }
                    break;
                }
            }
        }
    };
}

impl<'b> Bytes<'b> {
    #[inline]
    pub fn replace_byte(&self, repl: (u8, &[u8]), chunk_handler: &mut dyn FnMut(&Bytes<'_>)) {
        macro_rules! impls {
            (@find $tail:ident) => {
                memchr(repl.0, $tail)
            };

            (@get_replacement $tail:ident, $pos:ident) => {
                repl.1
            };
        }

        impl_replace_byte!(self, chunk_handler, impls);
    }

    #[inline]
    pub fn replace_byte2(
        &self,
        repl1: (u8, &[u8]),
        repl2: (u8, &[u8]),
        chunk_handler: &mut dyn FnMut(&Bytes<'_>),
    ) {
        macro_rules! impls {
            (@find $tail:ident) => {
                memchr2(repl1.0, repl2.0, $tail)
            };

            (@get_replacement $tail:ident, $pos:ident) => {{
                if $tail[$pos] == repl1.0 {
                    repl1.1
                } else {
                    repl2.1
                }
            }};
        }

        impl_replace_byte!(self, chunk_handler, impls);
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

impl_from_static!(0, 1, 2, 3, 4);

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
