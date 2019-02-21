use super::*;
use crate::base::Bytes;
use crate::parser::{
    Lexeme, NonTagContentLexeme, NonTagContentTokenOutline, TagLexeme, TagTokenOutline, TextType,
};
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

pub enum ToTokenResult<'i> {
    Token(Box<Token<'i>>),
    Text(TextType),
    None,
}

impl<'i> From<Token<'i>> for ToTokenResult<'i> {
    #[inline]
    fn from(token: Token<'i>) -> Self {
        ToTokenResult::Token(Box::new(token))
    }
}

pub trait ToToken {
    fn to_token(
        &self,
        capture_flags: TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> ToTokenResult<'_>;
}

impl ToToken for TagLexeme<'_> {
    fn to_token(
        &self,
        capture_flags: TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> ToTokenResult<'_> {
        match *self.token_outline() {
            TagTokenOutline::StartTag {
                name,
                ref attributes,
                self_closing,
                ..
            } if capture_flags.contains(TokenCaptureFlags::START_TAGS) => StartTag::new_token(
                self.part(name),
                Attributes::new(self.input(), Rc::clone(attributes), encoding),
                self_closing,
                self.raw(),
                encoding,
            )
            .into(),

            TagTokenOutline::EndTag { name, .. }
                if capture_flags.contains(TokenCaptureFlags::END_TAGS) =>
            {
                EndTag::new_token(self.part(name), self.raw(), encoding).into()
            }
            _ => ToTokenResult::None,
        }
    }
}

impl ToToken for NonTagContentLexeme<'_> {
    fn to_token(
        &self,
        capture_flags: TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> ToTokenResult<'_> {
        match *self.token_outline() {
            Some(NonTagContentTokenOutline::Text(text_type)) => ToTokenResult::Text(text_type),
            Some(NonTagContentTokenOutline::Comment(text))
                if capture_flags.contains(TokenCaptureFlags::COMMENTS) =>
            {
                Comment::new_token(self.part(text), self.raw(), encoding).into()
            }

            Some(NonTagContentTokenOutline::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            }) if capture_flags.contains(TokenCaptureFlags::DOCTYPES) => Doctype::new_token(
                self.opt_part(name),
                self.opt_part(public_id),
                self.opt_part(system_id),
                force_quirks,
                self.raw(),
                encoding,
            )
            .into(),
            _ => ToTokenResult::None,
        }
    }
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
                let token =
                    TextChunk::new_token(&buffer[..written], self.last_text_type, last, encoding);

                event_handler(TokenCapturerEvent::TokenProduced(Box::new(token)));
            }

            if let CoderResult::InputEmpty = status {
                break;
            }
        }
    }

    pub fn feed<'i, T>(
        &mut self,
        lexeme: &Lexeme<'i, T>,
        event_handler: &mut dyn FnMut(TokenCapturerEvent<'_>),
    ) where
        Lexeme<'i, T>: ToToken,
    {
        match lexeme.to_token(self.capture_flags, self.encoding) {
            ToTokenResult::Token(token) => {
                self.flush_pending_text(event_handler);
                event_handler(TokenCapturerEvent::LexemeConsumed);
                event_handler(TokenCapturerEvent::TokenProduced(token));
            }
            ToTokenResult::Text(text_type) => {
                if self.capture_flags.contains(TokenCaptureFlags::TEXT) {
                    self.last_text_type = text_type;

                    event_handler(TokenCapturerEvent::LexemeConsumed);

                    self.emit_text(&lexeme.raw(), false, event_handler);
                }
            }
            ToTokenResult::None => self.flush_pending_text(event_handler),
        }
    }
}
