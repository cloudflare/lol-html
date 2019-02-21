use super::*;
use crate::base::Bytes;
use crate::parser::{Lexeme, TextType, TokenOutline};
use bitflags::bitflags;
use encoding_rs::{CoderResult, Decoder, Encoding};
use std::rc::Rc;

pub const CAPTURE_TEXT: u8 = 0b0000_0001;
pub const CAPTURE_COMMENTS: u8 = 0b0000_0010;
pub const CAPTURE_START_TAGS: u8 = 0b0000_0100;
pub const CAPTURE_END_TAGS: u8 = 0b0000_1000;
pub const CAPTURE_DOCTYPES: u8 = 0b0001_0000;

bitflags! {
    pub struct TokenCaptureFlags: u8 {
        const TEXT = CAPTURE_TEXT;
        const COMMENTS = CAPTURE_COMMENTS;
        const START_TAGS = CAPTURE_START_TAGS;
        const END_TAGS = CAPTURE_END_TAGS;
        const DOCTYPES = CAPTURE_DOCTYPES;
    }
}

#[derive(Debug)]
pub enum TokenCapturerEvent<'i> {
    LexemeConsumed,
    TokenProduced(Box<Token<'i>>),
}

pub struct TokenCapturer {
    encoding: &'static Encoding,
    pending_text_decoder: Option<Decoder>,
    text_buffer: String,
    capture_flags: TokenCaptureFlags,
    document_level_capture_flags: TokenCaptureFlags,
    last_text_type: TextType,
}

impl TokenCapturer {
    pub fn new(
        document_level_capture_flags: TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> Self {
        TokenCapturer {
            encoding,
            pending_text_decoder: None,
            // TODO make adjustable
            text_buffer: String::from_utf8(vec![0u8; 1024]).unwrap(),
            capture_flags: document_level_capture_flags,
            document_level_capture_flags,
            last_text_type: TextType::Data,
        }
    }

    #[inline]
    pub fn has_captures(&self) -> bool {
        !self.capture_flags.is_empty()
    }

    #[inline]
    pub fn set_capture_flags(&mut self, flags: TokenCaptureFlags) {
        self.capture_flags = self.document_level_capture_flags | flags;
    }

    #[inline]
    pub fn stop_capturing_tags(&mut self) {
        self.capture_flags
            .remove(TokenCaptureFlags::START_TAGS | TokenCaptureFlags::END_TAGS);
    }

    #[inline]
    fn flush_pending_text(&mut self, event_handler: &mut dyn FnMut(TokenCapturerEvent<'_>)) {
        if self.pending_text_decoder.is_some() {
            self.emit_text(&Bytes::empty(), true, event_handler);
            self.pending_text_decoder = None;
        }
    }

    fn emit_text(
        &mut self,
        raw: &Bytes<'_>,
        last: bool,
        event_handler: &mut dyn FnMut(TokenCapturerEvent<'_>),
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

                event_handler(TokenCapturerEvent::TokenProduced(Box::new(token)));
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
        event_handler: &mut dyn FnMut(TokenCapturerEvent<'_>),
    ) {
        macro_rules! capture {
            ( $Type:ident ($($args:expr),+) ) => {
                event_handler(TokenCapturerEvent::LexemeConsumed);

                let token = Token::$Type($Type::new(
                    $($args),+,
                    lexeme.raw(),
                    self.encoding
                ));

                event_handler(TokenCapturerEvent::TokenProduced(Box::new(token)));
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
        event_handler: &mut dyn FnMut(TokenCapturerEvent<'_>),
    ) {
        match lexeme.token_outline() {
            Some(token_outline) => match *token_outline {
                TokenOutline::Text(text_type)
                    if self.capture_flags.contains(TokenCaptureFlags::TEXT) =>
                {
                    self.last_text_type = text_type;

                    event_handler(TokenCapturerEvent::LexemeConsumed);

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
