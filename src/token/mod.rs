mod capture;
mod impls;

use encoding_rs::Encoding;
use failure::Error;

pub use self::capture::{TokenCapture, TokenCaptureEvent, TokenCaptureFlags};
pub use self::impls::*;

pub trait Serialize {
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8]));
}

impl<T: Serialize> Serialize for Vec<T> {
    #[inline]
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8])) {
        for item in self {
            item.to_bytes(output_handler);
        }
    }
}

#[derive(Debug)]
pub enum Token<'i> {
    TextChunk(TextChunk<'i>),
    Comment(Comment<'i>),
    StartTag(StartTag<'i>),
    EndTag(EndTag<'i>),
    Doctype(Doctype<'i>),
    Eof,
}

impl<'i> Token<'i> {
    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> Token<'static> {
        match self {
            Token::TextChunk(t) => Token::TextChunk(t.to_owned()),
            Token::Comment(t) => Token::Comment(t.to_owned()),
            Token::StartTag(t) => Token::StartTag(t.to_owned()),
            Token::EndTag(t) => Token::EndTag(t.to_owned()),
            Token::Doctype(t) => Token::Doctype(t.to_owned()),
            Token::Eof => Token::Eof,
        }
    }
}

macro_rules! impl_from {
    ($($Type:ident),+) => {
        $(
            impl<'i> From<$Type<'i>> for Token<'i> {
                fn from(token: $Type<'i>) -> Self {
                    Token::$Type(token)
                }
            }
        )+
    };
}

impl_from!(TextChunk, Comment, StartTag, EndTag, Doctype);

impl Serialize for Token<'_> {
    #[inline]
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8])) {
        match self {
            Token::TextChunk(t) => t.to_bytes(output_handler),
            Token::Comment(t) => t.to_bytes(output_handler),
            Token::StartTag(t) => t.to_bytes(output_handler),
            Token::EndTag(t) => t.to_bytes(output_handler),
            Token::Doctype(t) => t.to_bytes(output_handler),
            Token::Eof => (),
        }
    }
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

    #[inline]
    pub fn new_text<'t>(&self, text: &'t str) -> TextChunk<'t> {
        TextChunk::new(text, self.encoding)
    }

    #[inline]
    pub fn try_script_text_from<'t>(&self, text: &'t str) -> Result<TextChunk<'t>, Error> {
        TextChunk::try_script_from(text, self.encoding)
    }

    #[inline]
    pub fn try_stylesheet_text_from<'t>(&self, text: &'t str) -> Result<TextChunk<'t>, Error> {
        TextChunk::try_stylesheet_from(text, self.encoding)
    }
}
