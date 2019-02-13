mod capture;
mod impls;

pub use self::capture::{TokenCapture, TokenCaptureEvent, TokenCaptureFlags};
pub use self::impls::*;

pub trait Serialize {
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8]));
}

#[derive(Debug)]
pub enum Token<'i> {
    TextChunk(TextChunk<'i>),
    Comment(Comment<'i>),
    StartTag(StartTag<'i>),
    EndTag(EndTag<'i>),
    Doctype(Doctype<'i>),
}

impl Serialize for Token<'_> {
    #[inline]
    fn to_bytes(&self, output_handler: &mut dyn FnMut(&[u8])) {
        match self {
            Token::TextChunk(t) => t.to_bytes(output_handler),
            Token::Comment(t) => t.to_bytes(output_handler),
            Token::StartTag(t) => t.to_bytes(output_handler),
            Token::EndTag(t) => t.to_bytes(output_handler),
            Token::Doctype(t) => t.to_bytes(output_handler),
        }
    }
}
