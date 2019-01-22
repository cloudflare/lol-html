mod capture;
mod impls;

use encoding_rs::Encoding;
use failure::Error;

pub use self::capture::{TokenCapture, TokenCaptureFlags, TokenCaptureResult};
pub use self::impls::*;

#[derive(Debug)]
pub enum Token<'i> {
    Text(TextChunk<'i>),
    Comment(Comment<'i>),
    StartTag(StartTag<'i>),
    EndTag(EndTag<'i>),
    Doctype(Doctype<'i>),
    Eof,
}

pub struct TokenFactory {
    encoding: &'static Encoding,
}

impl TokenFactory {
    pub fn new(encoding: &'static Encoding) -> Self {
        TokenFactory { encoding }
    }

    #[inline]
    pub fn try_start_tag_from(
        &self,
        name: &str,
        attributes: &[(&str, &str)],
        self_closing: bool,
    ) -> Result<StartTag<'static>, Error> {
        StartTag::try_from(name, attributes, self_closing, self.encoding)
    }

    #[inline]
    pub fn try_end_tag_from(&self, name: &str) -> Result<EndTag<'static>, Error> {
        EndTag::try_from(name, self.encoding)
    }

    #[inline]
    pub fn try_comment_from(&self, text: &str) -> Result<Comment<'static>, Error> {
        Comment::try_from(text, self.encoding)
    }
}
