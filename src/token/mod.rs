mod comment;
mod doctype;
mod end_tag;
mod start_tag;
mod text;

use crate::base::Bytes;
use crate::lexer::{Lexeme, TextType, TokenOutline};
use bitflags::bitflags;
use encoding_rs::{CoderResult, Decoder, Encoding};
use std::rc::Rc;

pub use self::comment::Comment;
pub use self::doctype::Doctype;
pub use self::end_tag::EndTag;
pub use self::start_tag::*;
pub use self::text::Text;

pub trait Serialize {
    fn serialize(&self) -> Bytes<'_>;
}

#[derive(Debug)]
pub enum Token<'i> {
    Text(Text<'i>),
    Comment(Comment<'i>),
    StartTag(StartTag<'i>),
    EndTag(EndTag<'i>),
    Doctype(Doctype<'i>),
    Eof,
}

bitflags! {
    pub struct TokenCaptureFlags: u8 {
        const TEXT = 0b0000_0001;
        const COMMENTS = 0b0000_0010;
        const START_TAGS = 0b0000_0100;
        const END_TAGS = 0b0000_1000;
        const DOCTYPES = 0b0001_0000;
        const EOF = 0b0010_0000;
    }
}

pub enum TokenCaptureResult<'i> {
    Captured(Token<'i>),
    Skipped(&'i Lexeme<'i>),
}

pub struct TokenCapture {
    encoding: &'static Encoding,
    pending_text_decoder: Option<Decoder>,
    text_buffer: String,
    capture_flags: TokenCaptureFlags,
    last_text_type: TextType,
}

impl TokenCapture {
    pub fn new(initial_capture_flags: TokenCaptureFlags, encoding: &'static Encoding) -> Self {
        TokenCapture {
            encoding,
            pending_text_decoder: None,
            // TODO make adjustable
            text_buffer: String::from_utf8(vec![0u8; 1024]).unwrap(),
            capture_flags: initial_capture_flags,
            last_text_type: TextType::Data,
        }
    }

    #[inline]
    fn flush_pending_text(&mut self, result_handler: &mut dyn FnMut(TokenCaptureResult<'_>)) {
        if self.pending_text_decoder.is_some() {
            self.emit_text(&Bytes::empty(), true, result_handler);
            self.pending_text_decoder = None;

            let result = TokenCaptureResult::Captured(Token::Text(Text::End));

            trace!(@output result);

            result_handler(result);
        }
    }

    fn emit_text(
        &mut self,
        raw: &Bytes<'_>,
        last: bool,
        result_handler: &mut dyn FnMut(TokenCaptureResult<'_>),
    ) {
        let encoding = self.encoding;
        let buffer = self.text_buffer.as_mut_str();

        let decoder = self
            .pending_text_decoder
            .get_or_insert_with(|| encoding.new_decoder_without_bom_handling());

        let mut consumed = 0usize;

        loop {
            let (result, read, written, ..) = decoder.decode_to_str(&raw[consumed..], buffer, last);

            consumed += read;

            if written > 0 {
                let chunk =
                    Text::new_parsed_chunk(&buffer[..written], self.last_text_type, encoding);

                let result = TokenCaptureResult::Captured(Token::Text(chunk));

                trace!(@output result);

                result_handler(result);
            }

            if let CoderResult::InputEmpty = result {
                break;
            }
        }
    }

    fn handle_non_textual_content(
        &mut self,
        lexeme: &Lexeme<'_>,
        token_outline: &TokenOutline,
        result_handler: &mut dyn FnMut(TokenCaptureResult<'_>),
    ) {
        macro_rules! capture {
            ( $Type:ident ($($args:expr),+) ) => {
                TokenCaptureResult::Captured(Token::$Type($Type::new_parsed(
                    $($args),+,
                    lexeme.raw(),
                    self.encoding
                )))
            };
        }

        let result = match token_outline {
            &TokenOutline::Comment(text)
                if self.capture_flags.contains(TokenCaptureFlags::COMMENTS) =>
            {
                capture!(Comment(lexeme.part(text)))
            }

            &TokenOutline::StartTag {
                name,
                ref attributes,
                self_closing,
                ..
            } if self.capture_flags.contains(TokenCaptureFlags::START_TAGS) => {
                let attributes =
                    ParsedAttributeList::new(lexeme.input(), Rc::clone(attributes), self.encoding);

                capture!(StartTag(lexeme.part(name), attributes, self_closing))
            }

            &TokenOutline::EndTag { name, .. }
                if self.capture_flags.contains(TokenCaptureFlags::END_TAGS) =>
            {
                capture!(EndTag(lexeme.part(name)))
            }

            &TokenOutline::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            } if self.capture_flags.contains(TokenCaptureFlags::DOCTYPES) => capture!(Doctype(
                lexeme.opt_part(name),
                lexeme.opt_part(public_id),
                lexeme.opt_part(system_id),
                force_quirks
            )),

            TokenOutline::Eof if self.capture_flags.contains(TokenCaptureFlags::EOF) => {
                TokenCaptureResult::Captured(Token::Eof)
            }
            _ => TokenCaptureResult::Skipped(lexeme),
        };

        trace!(@output result);

        result_handler(result);
    }

    pub fn feed(
        &mut self,
        lexeme: &Lexeme<'_>,
        result_handler: &mut dyn FnMut(TokenCaptureResult<'_>),
    ) {
        match lexeme.token_outline() {
            Some(token_outline) => match *token_outline {
                TokenOutline::Text(text_type)
                    if self.capture_flags.contains(TokenCaptureFlags::TEXT) =>
                {
                    self.last_text_type = text_type;
                    self.emit_text(&lexeme.raw(), false, result_handler);
                }
                TokenOutline::Text(_) => result_handler(TokenCaptureResult::Skipped(lexeme)),
                _ => {
                    self.flush_pending_text(result_handler);
                    self.handle_non_textual_content(lexeme, token_outline, result_handler);
                }
            },

            None => {
                self.flush_pending_text(result_handler);
                result_handler(TokenCaptureResult::Skipped(lexeme));
            }
        }
    }
}
