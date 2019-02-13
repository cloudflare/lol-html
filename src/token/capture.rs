use super::*;
use crate::base::Bytes;
use crate::parser::{Lexeme, TextType, TokenOutline};
use bitflags::bitflags;
use encoding_rs::{CoderResult, Decoder, Encoding};
use std::rc::Rc;

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

#[derive(Debug)]
pub enum TokenCaptureEvent<'i> {
    LexemeConsumed,
    TokenProduced(Box<Token<'i>>),
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
    fn flush_pending_text(&mut self, event_handler: &mut dyn FnMut(TokenCaptureEvent<'_>)) {
        if self.pending_text_decoder.is_some() {
            self.emit_text(&Bytes::empty(), true, event_handler);
            self.pending_text_decoder = None;
        }
    }

    fn emit_text(
        &mut self,
        raw: &Bytes<'_>,
        last: bool,
        event_handler: &mut dyn FnMut(TokenCaptureEvent<'_>),
    ) {
        let encoding = self.encoding;
        let buffer = self.text_buffer.as_mut_str();

        let decoder = self
            .pending_text_decoder
            .get_or_insert_with(|| encoding.new_decoder_without_bom_handling());

        let mut consumed = 0usize;

        loop {
            let (status, read, written, ..) = decoder.decode_to_str(&raw[consumed..], buffer, last);

            consumed += read;

            if written > 0 || last {
                let token = Token::TextChunk(TextChunk::new(
                    &buffer[..written],
                    self.last_text_type,
                    last,
                    encoding,
                ));

                event_handler(TokenCaptureEvent::TokenProduced(Box::new(token)));
            }

            if let CoderResult::InputEmpty = status {
                break;
            }
        }
    }

    fn handle_non_textual_content(
        &mut self,
        lexeme: &Lexeme<'_>,
        token_outline: &TokenOutline,
        event_handler: &mut dyn FnMut(TokenCaptureEvent<'_>),
    ) {
        macro_rules! capture {
            ( $Type:ident ($($args:expr),+) ) => {
                event_handler(TokenCaptureEvent::LexemeConsumed);

                let token = Token::$Type($Type::new(
                    $($args),+,
                    lexeme.raw(),
                    self.encoding
                ));

                event_handler(TokenCaptureEvent::TokenProduced(Box::new(token)));
            };
        }

        match *token_outline {
            TokenOutline::Comment(text)
                if self.capture_flags.contains(TokenCaptureFlags::COMMENTS) =>
            {
                capture!(Comment(lexeme.part(text)));
            }

            TokenOutline::StartTag {
                name,
                ref attributes,
                self_closing,
                ..
            } if self.capture_flags.contains(TokenCaptureFlags::START_TAGS) => {
                capture!(StartTag(
                    lexeme.part(name),
                    Attributes::new(lexeme.input(), Rc::clone(attributes), self.encoding),
                    self_closing
                ));
            }

            TokenOutline::EndTag { name, .. }
                if self.capture_flags.contains(TokenCaptureFlags::END_TAGS) =>
            {
                capture!(EndTag(lexeme.part(name)));
            }

            TokenOutline::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            } if self.capture_flags.contains(TokenCaptureFlags::DOCTYPES) => {
                capture!(Doctype(
                    lexeme.opt_part(name),
                    lexeme.opt_part(public_id),
                    lexeme.opt_part(system_id),
                    force_quirks
                ));
            }
            _ => (),
        }
    }

    pub fn feed(
        &mut self,
        lexeme: &Lexeme<'_>,
        event_handler: &mut dyn FnMut(TokenCaptureEvent<'_>),
    ) {
        match lexeme.token_outline() {
            Some(token_outline) => match *token_outline {
                TokenOutline::Text(text_type)
                    if self.capture_flags.contains(TokenCaptureFlags::TEXT) =>
                {
                    self.last_text_type = text_type;

                    event_handler(TokenCaptureEvent::LexemeConsumed);

                    self.emit_text(&lexeme.raw(), false, event_handler);
                }
                TokenOutline::Text(_) => (),
                _ => {
                    self.flush_pending_text(event_handler);
                    self.handle_non_textual_content(lexeme, token_outline, event_handler);
                }
            },
            None => self.flush_pending_text(event_handler),
        }
    }
}
