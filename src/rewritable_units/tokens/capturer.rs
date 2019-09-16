use super::*;
use crate::base::Bytes;
use crate::html::TextType;
use crate::parser::{
    Lexeme, NonTagContentLexeme, NonTagContentTokenOutline, TagLexeme, TagTokenOutline,
};
use crate::rewriter::RewritingError;
use bitflags::bitflags;
use encoding_rs::{CoderResult, Decoder, Encoding};
use std::rc::Rc;

bitflags! {
    pub struct TokenCaptureFlags: u8 {
        const TEXT = 0b0000_0001;
        const COMMENTS = 0b0000_0010;
        const NEXT_START_TAG = 0b0000_0100;
        const NEXT_END_TAG = 0b0000_1000;
        const DOCTYPES = 0b0001_0000;
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
        capture_flags: &mut TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> ToTokenResult;
}

impl ToToken for TagLexeme<'_> {
    fn to_token(
        &self,
        capture_flags: &mut TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> ToTokenResult {
        match *self.token_outline() {
            TagTokenOutline::StartTag {
                name,
                ref attributes,
                ns,
                self_closing,
                ..
            } if capture_flags.contains(TokenCaptureFlags::NEXT_START_TAG) => {
                // NOTE: clear the flag once we've seen required start tag.
                capture_flags.remove(TokenCaptureFlags::NEXT_START_TAG);

                StartTag::new_token(
                    self.part(name),
                    Attributes::new(self.input(), Rc::clone(attributes), encoding),
                    ns,
                    self_closing,
                    self.raw(),
                    encoding,
                )
                .into()
            }

            TagTokenOutline::EndTag { name, .. }
                if capture_flags.contains(TokenCaptureFlags::NEXT_END_TAG) =>
            {
                // NOTE: clear the flag once we've seen required end tag.
                capture_flags.remove(TokenCaptureFlags::NEXT_END_TAG);

                EndTag::new_token(self.part(name), self.raw(), encoding).into()
            }
            _ => ToTokenResult::None,
        }
    }
}

impl ToToken for NonTagContentLexeme<'_> {
    fn to_token(
        &self,
        capture_flags: &mut TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> ToTokenResult {
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
    last_text_type: TextType,
}

impl TokenCapturer {
    pub fn new(capture_flags: TokenCaptureFlags, encoding: &'static Encoding) -> Self {
        TokenCapturer {
            encoding,
            pending_text_decoder: None,
            // TODO make adjustable
            text_buffer: String::from_utf8(vec![0u8; 1024]).unwrap(),
            capture_flags,
            last_text_type: TextType::Data,
        }
    }

    #[inline]
    pub fn has_captures(&self) -> bool {
        !self.capture_flags.is_empty()
    }

    #[inline]
    pub fn set_capture_flags(&mut self, flags: TokenCaptureFlags) {
        self.capture_flags = flags;
    }

    #[inline]
    pub fn flush_pending_text(
        &mut self,
        event_handler: &mut dyn FnMut(TokenCapturerEvent) -> Result<(), RewritingError>,
    ) -> Result<(), RewritingError> {
        if self.pending_text_decoder.is_some() {
            self.emit_text(&Bytes::empty(), true, event_handler)?;
            self.pending_text_decoder = None;
        }

        Ok(())
    }

    fn emit_text(
        &mut self,
        raw: &Bytes,
        last: bool,
        event_handler: &mut dyn FnMut(TokenCapturerEvent) -> Result<(), RewritingError>,
    ) -> Result<(), RewritingError> {
        let encoding = self.encoding;
        let buffer = self.text_buffer.as_mut_str();

        let decoder = self
            .pending_text_decoder
            .get_or_insert_with(|| encoding.new_decoder_without_bom_handling());

        let mut consumed = 0;

        loop {
            let (status, read, written, ..) = decoder.decode_to_str(&raw[consumed..], buffer, last);

            consumed += read;

            if written > 0 || last {
                let token =
                    TextChunk::new_token(&buffer[..written], self.last_text_type, last, encoding);

                event_handler(TokenCapturerEvent::TokenProduced(Box::new(token)))?;
            }

            if let CoderResult::InputEmpty = status {
                break;
            }
        }

        Ok(())
    }

    pub fn feed<'i, T>(
        &mut self,
        lexeme: &Lexeme<'i, T>,
        mut event_handler: impl FnMut(TokenCapturerEvent) -> Result<(), RewritingError>,
    ) -> Result<(), RewritingError>
    where
        Lexeme<'i, T>: ToToken,
    {
        match lexeme.to_token(&mut self.capture_flags, self.encoding) {
            ToTokenResult::Token(token) => {
                self.flush_pending_text(&mut event_handler)?;
                event_handler(TokenCapturerEvent::LexemeConsumed)?;
                event_handler(TokenCapturerEvent::TokenProduced(token))
            }
            ToTokenResult::Text(text_type) => {
                if self.capture_flags.contains(TokenCaptureFlags::TEXT) {
                    self.last_text_type = text_type;

                    event_handler(TokenCapturerEvent::LexemeConsumed)?;

                    self.emit_text(&lexeme.raw(), false, &mut event_handler)?;
                }

                Ok(())
            }
            ToTokenResult::None => self.flush_pending_text(&mut event_handler),
        }
    }
}
