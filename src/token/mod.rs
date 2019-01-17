mod capture;
mod factory;
mod impls;

pub use self::capture::{TokenCapture, TokenCaptureFlags, TokenCaptureResult};
pub use self::factory::TokenFactory;
pub use self::impls::*;

#[derive(Debug)]
pub enum Token<'i> {
    Text(Text<'i>),
    Comment(Comment<'i>),
    StartTag(StartTag<'i>),
    EndTag(EndTag<'i>),
    Doctype(Doctype<'i>),
    Eof,
}
