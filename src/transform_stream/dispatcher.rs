use super::*;
use crate::base::{Chunk, Range};
use crate::html::{LocalName, Namespace};
use crate::parser::{
    Lexeme, LexemeSink, NonTagContentLexeme, ParserDirective, ParserOutputSink, TagHintSink,
    TagLexeme, TagTokenOutline,
};
use crate::rewritable_units::{
    Serialize, ToToken, Token, TokenCaptureFlags, TokenCapturer, TokenCapturerEvent,
};
use encoding_rs::Encoding;
use std::rc::Rc;

use TagTokenOutline::*;

pub struct AuxStartTagInfo<'i> {
    pub input: &'i Chunk<'i>,
    pub attr_buffer: SharedAttributeBuffer,
    pub self_closing: bool,
}

type AuxStartTagInfoRequest<C> = Box<dyn FnMut(&mut C, AuxStartTagInfo<'_>) -> TokenCaptureFlags>;
pub type StartTagHandlingResult<C> = Result<TokenCaptureFlags, AuxStartTagInfoRequest<C>>;

pub trait TransformController: Sized {
    fn initial_capture_flags(&self) -> TokenCaptureFlags;
    fn handle_start_tag(&mut self, name: LocalName, ns: Namespace) -> StartTagHandlingResult<Self>;
    fn handle_end_tag(&mut self, name: LocalName) -> TokenCaptureFlags;
    fn handle_token(&mut self, token: &mut Token);
}

pub trait OutputSink {
    fn handle_chunk(&mut self, chunk: &[u8]);
}

impl<F: FnMut(&[u8])> OutputSink for F {
    fn handle_chunk(&mut self, chunk: &[u8]) {
        self(chunk);
    }
}

pub struct Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    transform_controller: C,
    output_sink: O,
    last_consumed_lexeme_end: usize,
    token_capturer: TokenCapturer,
    got_flags_from_hint: bool,
    pending_element_aux_info_req: Option<AuxStartTagInfoRequest<C>>,
}

impl<C, O> Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    pub fn new(transform_controller: C, output_sink: O, encoding: &'static Encoding) -> Self {
        let initial_capture_flags = transform_controller.initial_capture_flags();

        Dispatcher {
            transform_controller,
            output_sink,
            last_consumed_lexeme_end: 0,
            token_capturer: TokenCapturer::new(initial_capture_flags, encoding),
            got_flags_from_hint: false,
            pending_element_aux_info_req: None,
        }
    }

    pub fn flush_remaining_input(&mut self, input: &Chunk, blocked_byte_count: usize) {
        let output = input.slice(Range {
            start: self.last_consumed_lexeme_end,
            end: input.len() - blocked_byte_count,
        });

        if !output.is_empty() {
            self.output_sink.handle_chunk(&output);
        }

        self.last_consumed_lexeme_end = 0;
    }

    pub fn finish(&mut self, input: &Chunk) {
        self.flush_remaining_input(input, 0);

        // NOTE: output the finalizing chunk.
        self.output_sink.handle_chunk(&[]);
    }

    fn try_produce_token_from_lexeme<'i, T>(&mut self, lexeme: &Lexeme<'i, T>)
    where
        Lexeme<'i, T>: ToToken,
    {
        let transform_controller = &mut self.transform_controller;
        let output_sink = &mut self.output_sink;
        let lexeme_range = lexeme.raw_range();
        let last_consumed_lexeme_end = self.last_consumed_lexeme_end;
        let mut lexeme_consumed = false;

        self.token_capturer.feed(lexeme, |event| match event {
            TokenCapturerEvent::LexemeConsumed => {
                let chunk = lexeme.input().slice(Range {
                    start: last_consumed_lexeme_end,
                    end: lexeme_range.start,
                });

                lexeme_consumed = true;

                if chunk.len() > 0 {
                    output_sink.handle_chunk(&chunk);
                }
            }
            TokenCapturerEvent::TokenProduced(mut token) => {
                trace!(@output token);

                transform_controller.handle_token(&mut token);
                token.to_bytes(&mut |c| output_sink.handle_chunk(c));
            }
        });

        if lexeme_consumed {
            self.last_consumed_lexeme_end = lexeme_range.end;
        }
    }

    #[inline]
    fn get_next_parser_directive(&self) -> ParserDirective {
        if self.token_capturer.has_captures() {
            ParserDirective::Lex
        } else {
            ParserDirective::WherePossibleScanForTagsOnly
        }
    }

    fn adjust_capture_flags_for_tag_lexeme(&mut self, lexeme: &TagLexeme) {
        let input = lexeme.input();

        macro_rules! get_flags_from_aux_info_res {
            ($handler:expr, $attributes:expr, $self_closing:expr) => {
                $handler(
                    &mut self.transform_controller,
                    AuxStartTagInfo {
                        input,
                        attr_buffer: Rc::clone($attributes),
                        self_closing: $self_closing,
                    },
                )
            };
        }

        let capture_flags = match self.pending_element_aux_info_req.take() {
            // NOTE: tag hint was produced for the tag, but
            // attributes and self closing flag were requested.
            Some(mut aux_info_req) => match *lexeme.token_outline() {
                StartTag {
                    ref attributes,
                    self_closing,
                    ..
                } => get_flags_from_aux_info_res!(aux_info_req, attributes, self_closing),
                _ => unreachable!("Tag should be a start tag at this point"),
            },

            // NOTE: tag hint hasn't been produced for the tag, because
            // parser is not in the tag scan mode.
            None => match *lexeme.token_outline() {
                StartTag {
                    name,
                    name_hash,
                    ns,
                    ref attributes,
                    self_closing,
                } => {
                    let name = LocalName::new(input, name, name_hash);

                    match self.transform_controller.handle_start_tag(name, ns) {
                        Ok(flags) => flags,
                        Err(mut aux_info_req) => {
                            get_flags_from_aux_info_res!(aux_info_req, attributes, self_closing)
                        }
                    }
                }

                EndTag { name, name_hash } => {
                    let name = LocalName::new(input, name, name_hash);

                    self.transform_controller.handle_end_tag(name)
                }
            },
        };

        self.token_capturer.set_capture_flags(capture_flags);
    }

    #[inline]
    fn apply_capture_flags_from_hint_and_get_next_parser_directive(
        &mut self,
        flags: TokenCaptureFlags,
    ) -> ParserDirective {
        self.token_capturer.set_capture_flags(flags);
        self.got_flags_from_hint = true;
        self.get_next_parser_directive()
    }
}

impl<C, O> LexemeSink for Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    fn handle_tag(&mut self, lexeme: &TagLexeme) -> ParserDirective {
        if self.got_flags_from_hint {
            self.got_flags_from_hint = false;
        } else {
            self.adjust_capture_flags_for_tag_lexeme(lexeme);
        }

        self.try_produce_token_from_lexeme(lexeme);
        self.get_next_parser_directive()
    }

    #[inline]
    fn handle_non_tag_content(&mut self, lexeme: &NonTagContentLexeme) {
        self.try_produce_token_from_lexeme(lexeme);
    }
}

impl<C, O> TagHintSink for Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    fn handle_start_tag_hint(&mut self, name: LocalName, ns: Namespace) -> ParserDirective {
        match self.transform_controller.handle_start_tag(name, ns) {
            Ok(flags) => self.apply_capture_flags_from_hint_and_get_next_parser_directive(flags),
            Err(aux_info_req) => {
                self.pending_element_aux_info_req = Some(aux_info_req);

                ParserDirective::Lex
            }
        }
    }

    fn handle_end_tag_hint(&mut self, name: LocalName) -> ParserDirective {
        let flags = self.transform_controller.handle_end_tag(name);

        self.apply_capture_flags_from_hint_and_get_next_parser_directive(flags)
    }
}

impl<C, O> ParserOutputSink for Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
}
