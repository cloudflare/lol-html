use crate::base::Bytes;
use crate::token::Token;
use crate::token::*;
use crate::tokenizer::{
    LexUnit, LexUnitSink, NextOutputType, OutputSink as TokenizerOutputSink, TagPreview,
    TagPreviewSink, TokenView,
};
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

// OutputSink
// handle_bailout
pub trait TransformController {
    fn get_token_capture_flags_for_tag(&mut self, tag_lex_unit: &LexUnit) -> NextOutputType;
    fn get_token_capture_flags_for_tag_preview(
        &mut self,
        tag_preview: &TagPreview,
    ) -> NextOutputType;

    fn handle_token(&mut self, token: Token);
}

pub struct Writer<C: TransformController> {
    transform_controller: C,
    encoding: &'static Encoding,
    pending_text_decoder: Option<Decoder>,
    text_buffer: String,
    token_capture_flags: TokenCaptureFlags,
}

impl<C: TransformController> Writer<C> {
    pub fn new(transform_controller: C, encoding: &'static Encoding) -> Self {
        Writer {
            transform_controller,
            encoding,
            pending_text_decoder: None,
            // TODO make adjustable
            text_buffer: String::from_utf8(vec![0u8; 1024]).unwrap(),
            token_capture_flags: TokenCaptureFlags::empty(),
        }
    }

    // TODO: temporary code that just toggles all flags depending on the next output type.
    fn adjust_token_capture_flags(&mut self, next_output_type: NextOutputType) {
        self.token_capture_flags = match next_output_type {
            NextOutputType::LexUnit => TokenCaptureFlags::all(),
            NextOutputType::TagPreview => TokenCaptureFlags::empty(),
        };
    }

    #[inline]
    fn flush_pending_text(&mut self) {
        if self.pending_text_decoder.is_some() {
            self.emit_text(Bytes::empty(), true);

            let token = Token::Text(Text::End);

            self.transform_controller.handle_token(token);
        }
    }

    fn emit_text(&mut self, raw: Bytes<'_>, last: bool) {
        let encoding = self.encoding;
        let buffer = self.text_buffer.as_mut_str();

        let decoder = self
            .pending_text_decoder
            .get_or_insert_with(|| encoding.new_decoder());

        let mut consumed = 0usize;

        loop {
            let (result, read, written, ..) = decoder.decode_to_str(&raw[consumed..], buffer, last);

            consumed += read;

            if written > 0 {
                let token = Token::Text(Text::new_parsed_chunk(&buffer[..written], encoding));

                self.transform_controller.handle_token(token);
            }

            if let CoderResult::InputEmpty = result {
                break;
            }
        }
    }

    fn handle_non_textual_content(&mut self, lex_unit: &LexUnit<'_>, token_view: &TokenView) {
        let token = match token_view {
            &TokenView::Comment(text)
                if self
                    .token_capture_flags
                    .contains(TokenCaptureFlags::COMMENTS) =>
            {
                Some(Token::Comment(Comment::new_parsed(
                    lex_unit.part(text),
                    lex_unit.raw(),
                    self.encoding,
                )))
            }

            &TokenView::StartTag {
                name,
                ref attributes,
                self_closing,
                ..
            } if self
                .token_capture_flags
                .contains(TokenCaptureFlags::START_TAGS) =>
            {
                Some(Token::StartTag(StartTag::new_parsed(
                    lex_unit.part(name),
                    ParsedAttributeList::new(
                        lex_unit.input(),
                        Rc::clone(attributes),
                        self.encoding,
                    ),
                    self_closing,
                    lex_unit.raw(),
                    self.encoding,
                )))
            }

            &TokenView::EndTag { name, .. }
                if self
                    .token_capture_flags
                    .contains(TokenCaptureFlags::END_TAGS) =>
            {
                Some(Token::EndTag(EndTag::new_parsed(
                    lex_unit.part(name),
                    lex_unit.raw(),
                    self.encoding,
                )))
            }

            &TokenView::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            } if self
                .token_capture_flags
                .contains(TokenCaptureFlags::DOCTYPES) =>
            {
                Some(Token::Doctype(Doctype::new_parsed(
                    lex_unit.opt_part(name),
                    lex_unit.opt_part(public_id),
                    lex_unit.opt_part(system_id),
                    force_quirks,
                    lex_unit.raw(),
                    self.encoding,
                )))
            }

            TokenView::Eof if self.token_capture_flags.contains(TokenCaptureFlags::EOF) => {
                Some(Token::Eof)
            }
            _ => None,
        };

        if let Some(token) = token {
            self.transform_controller.handle_token(token);
        }
    }

    fn handle_lex_unit(&mut self, lex_unit: &LexUnit<'_>) {
        if let Some(token_view) = lex_unit.token_view() {
            if let TokenView::Text = token_view {
                if self.token_capture_flags.contains(TokenCaptureFlags::TEXT) {
                    self.emit_text(lex_unit.raw(), false);
                }
            } else {
                self.flush_pending_text();
                self.handle_non_textual_content(lex_unit, token_view);
            }
        } else {
            self.flush_pending_text();
        }
    }
}

impl<C: TransformController> LexUnitSink for Writer<C> {
    #[inline]
    fn handle_tag(&mut self, lex_unit: &LexUnit<'_>) -> NextOutputType {
        let next_output_type = self
            .transform_controller
            .get_token_capture_flags_for_tag(lex_unit);

        self.adjust_token_capture_flags(next_output_type);
        self.handle_lex_unit(lex_unit);

        next_output_type
    }

    #[inline]
    fn handle_non_tag_content(&mut self, lex_unit: &LexUnit<'_>) {
        self.handle_lex_unit(lex_unit);
    }
}

impl<C: TransformController> TagPreviewSink for Writer<C> {
    #[inline]
    fn handle_tag_preview(&mut self, tag_preview: &TagPreview<'_>) -> NextOutputType {
        let next_output_type = self
            .transform_controller
            .get_token_capture_flags_for_tag_preview(tag_preview);

        self.adjust_token_capture_flags(next_output_type);

        next_output_type
    }
}

impl<C: TransformController> TokenizerOutputSink for Writer<C> {}
