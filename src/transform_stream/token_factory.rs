use crate::base::{Bytes, Chunk, Range};
use crate::token::Token;
use crate::token::*;
use crate::tokenizer::{
    AttributeView, LexUnit, LexUnitSink, NextOutputType, OutputSink as TokenizerOutputSink,
    TagPreview, TagPreviewSink, TokenView,
};
use bitflags::bitflags;
use encoding_rs::{CoderResult, Decoder, Encoding};
use std::cell::RefCell;
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

pub trait TokenRewriter {
    fn get_token_capture_flags_for_tag(&mut self, tag_lex_unit: &LexUnit) -> NextOutputType;
    fn get_token_capture_flags_for_tag_preview(
        &mut self,
        tag_preview: &TagPreview,
    ) -> NextOutputType;

    fn handle_token(&mut self, token: Token);
}

pub struct TokenFactory<R: TokenRewriter> {
    token_rewriter: R,
    encoding: &'static Encoding,
    pending_text_decoder: Option<Decoder>,
    text_buffer: String,
    token_production_flags: TokenCaptureFlags,
}

impl<R: TokenRewriter> TokenFactory<R> {
    // TODO: temporary code that just toggles all flags depending on the next output type.
    fn adjust_token_production_flags(&mut self, next_output_type: NextOutputType) {
        self.token_production_flags = match next_output_type {
            NextOutputType::LexUnit => TokenCaptureFlags::all(),
            NextOutputType::TagPreview => TokenCaptureFlags::empty(),
        };
    }

    // TODO tokenizer output sync trait
    fn handle_text(&mut self, raw: Bytes<'_>) {
        let encoding = self.encoding;
        let buffer = self.text_buffer.as_mut_str();

        let decoder = self
            .pending_text_decoder
            .get_or_insert_with(|| encoding.new_decoder());

        let mut consumed = 0usize;

        loop {
            let (result, read, written, ..) =
                decoder.decode_to_str(&raw[consumed..], buffer, false);

            consumed += read;

            if written > 0 {
                let token = Token::Text(Text::new_parsed_chunk(&buffer[..written], encoding));

                self.token_rewriter.handle_token(token);
            }

            if let CoderResult::InputEmpty = result {
                break;
            }
        }
    }

    fn handle_comment(&mut self, text: Range, raw: Bytes<'_>, input: &Chunk<'_>) {
        let text = input.slice(text);
        let token = Token::Comment(Comment::new_parsed(text, raw, self.encoding));

        self.token_rewriter.handle_token(token);
    }

    fn handle_start_tag(
        &mut self,
        name: Range,
        attributes: &Rc<RefCell<Vec<AttributeView>>>,
        self_closing: bool,
        raw: Bytes<'_>,
        input: &Chunk<'_>,
    ) {
        let name = input.slice(name);
        let attributes = ParsedAttributeList::new(input, Rc::clone(attributes), self.encoding);

        let token = Token::StartTag(StartTag::new_parsed(
            name,
            attributes,
            self_closing,
            raw,
            self.encoding,
        ));

        self.token_rewriter.handle_token(token);
    }

    fn handle_end_tag(&mut self, name: Range, raw: Bytes<'_>, input: &Chunk<'_>) {
        let name = input.slice(name);
        let token = Token::EndTag(EndTag::new_parsed(name, raw, self.encoding));

        self.token_rewriter.handle_token(token);
    }

    fn handle_doctype(
        &mut self,
        name: Option<Range>,
        public_id: Option<Range>,
        system_id: Option<Range>,
        force_quirks: bool,
        raw: Bytes<'_>,
        input: &Chunk<'_>,
    ) {
        let name = input.opt_slice(name);
        let public_id = input.opt_slice(public_id);
        let system_id = input.opt_slice(system_id);

        let token = Token::Doctype(Doctype::new_parsed(
            name,
            public_id,
            system_id,
            force_quirks,
            raw,
            self.encoding,
        ));

        self.token_rewriter.handle_token(token);
    }

    fn handle_eof(&mut self) {
        self.token_rewriter.handle_token(Token::Eof);
    }

    fn handle_lex_unit(&mut self, lex_unit: &LexUnit<'_>) {
        if let Some(token_view) = lex_unit.token_view() {
            let input = lex_unit.input();
            let raw = input.slice(lex_unit.raw_range());

            match token_view {
                TokenView::Text => self.handle_text(raw),
                &TokenView::Comment(text) => self.handle_comment(text, raw, input),

                &TokenView::StartTag {
                    name,
                    ref attributes,
                    self_closing,
                    ..
                } => self.handle_start_tag(name, attributes, self_closing, raw, input),

                &TokenView::EndTag { name, .. } => self.handle_end_tag(name, raw, input),

                &TokenView::Doctype {
                    name,
                    public_id,
                    system_id,
                    force_quirks,
                } => self.handle_doctype(name, public_id, system_id, force_quirks, raw, input),

                TokenView::Eof => self.handle_eof(),
            }
        }
    }
}

impl<R: TokenRewriter> LexUnitSink for TokenFactory<R> {
    #[inline]
    fn handle_tag(&mut self, lex_unit: &LexUnit<'_>) -> NextOutputType {
        let next_output_type = self
            .token_rewriter
            .get_token_capture_flags_for_tag(lex_unit);

        self.adjust_token_production_flags(next_output_type);
        self.handle_lex_unit(lex_unit);

        next_output_type
    }

    #[inline]
    fn handle_non_tag_content(&mut self, lex_unit: &LexUnit<'_>) {
        self.handle_lex_unit(lex_unit);
    }
}

impl<R: TokenRewriter> TagPreviewSink for TokenFactory<R> {
    #[inline]
    fn handle_tag_preview(&mut self, tag_preview: &TagPreview<'_>) -> NextOutputType {
        let next_output_type = self
            .token_rewriter
            .get_token_capture_flags_for_tag_preview(tag_preview);

        self.adjust_token_production_flags(next_output_type);

        next_output_type
    }
}

impl<R: TokenRewriter> TokenizerOutputSink for TokenFactory<R> {}
