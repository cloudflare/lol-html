mod comment;
mod doctype;
mod end_tag;
mod start_tag;
mod text;

use crate::base::Bytes;
use crate::tokenizer::{LexUnit, TextType, TokenView};
use bitflags::bitflags;
use encoding_rs::{CoderResult, Decoder, Encoding};
use std::rc::Rc;

pub use self::comment::Comment;
pub use self::doctype::Doctype;
pub use self::end_tag::EndTag;
pub use self::start_tag::*;
pub use self::text::Text;

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
    Skipped(&'i LexUnit<'i>),
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
        lex_unit: &LexUnit<'_>,
        token_view: &TokenView,
        result_handler: &mut dyn FnMut(TokenCaptureResult<'_>),
    ) {
        macro_rules! capture {
            ( $Type:ident ($($args:expr),+) ) => {
                TokenCaptureResult::Captured(Token::$Type($Type::new_parsed(
                    $($args),+,
                    lex_unit.raw(),
                    self.encoding
                )))
            };
        }

        let result = match token_view {
            &TokenView::Comment(text)
                if self.capture_flags.contains(TokenCaptureFlags::COMMENTS) =>
            {
                capture!(Comment(lex_unit.part(text)))
            }

            &TokenView::StartTag {
                name,
                ref attributes,
                self_closing,
                ..
            } if self.capture_flags.contains(TokenCaptureFlags::START_TAGS) => {
                let attributes = ParsedAttributeList::new(
                    lex_unit.input(),
                    Rc::clone(attributes),
                    self.encoding,
                );

                capture!(StartTag(lex_unit.part(name), attributes, self_closing))
            }

            &TokenView::EndTag { name, .. }
                if self.capture_flags.contains(TokenCaptureFlags::END_TAGS) =>
            {
                capture!(EndTag(lex_unit.part(name)))
            }

            &TokenView::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            } if self.capture_flags.contains(TokenCaptureFlags::DOCTYPES) => capture!(Doctype(
                lex_unit.opt_part(name),
                lex_unit.opt_part(public_id),
                lex_unit.opt_part(system_id),
                force_quirks
            )),

            TokenView::Eof if self.capture_flags.contains(TokenCaptureFlags::EOF) => {
                TokenCaptureResult::Captured(Token::Eof)
            }
            _ => TokenCaptureResult::Skipped(lex_unit),
        };

        trace!(@output result);

        result_handler(result);
    }

    pub fn feed(
        &mut self,
        lex_unit: &LexUnit<'_>,
        result_handler: &mut dyn FnMut(TokenCaptureResult<'_>),
    ) {
        match lex_unit.token_view() {
            Some(token_view) => match *token_view {
                TokenView::Text(text_type)
                    if self.capture_flags.contains(TokenCaptureFlags::TEXT) =>
                {
                    self.last_text_type = text_type;
                    self.emit_text(&lex_unit.raw(), false, result_handler);
                }
                TokenView::Text(_) => result_handler(TokenCaptureResult::Skipped(lex_unit)),
                _ => {
                    self.flush_pending_text(result_handler);
                    self.handle_non_textual_content(lex_unit, token_view, result_handler);
                }
            },

            None => {
                self.flush_pending_text(result_handler);
                result_handler(TokenCaptureResult::Skipped(lex_unit));
            }
        }
    }
}
