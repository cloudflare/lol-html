use super::transform_controller::*;
use crate::base::{Chunk, Range};
use crate::parser::{
    Lexeme, LexemeSink, NextOutputType, ParserOutputSink, TagHint, TagHintSink, TagNameInfo,
    TokenOutline,
};
use crate::token::{Serialize, TokenCapturer, TokenCapturerEvent};
use encoding_rs::Encoding;
use std::rc::Rc;

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
    token_capture: TokenCapturer,
    got_capture_flags_from_tag_hint: bool,
    pending_element_modifiers_info_handler: Option<ElementModifiersInfoHandler<C>>,
}

impl<C, O> Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    pub fn new(transform_controller: C, output_sink: O, encoding: &'static Encoding) -> Self {
        let initial_capture_flags = transform_controller
            .document_level_content_settings()
            .into();

        Dispatcher {
            transform_controller,
            output_sink,
            last_consumed_lexeme_end: 0,
            token_capture: TokenCapturer::new(initial_capture_flags, encoding),
            got_capture_flags_from_tag_hint: false,
            pending_element_modifiers_info_handler: None,
        }
    }

    pub fn flush_remaining_input(&mut self, input: &Chunk<'_>, blocked_byte_count: usize) {
        let output = input.slice(Range {
            start: self.last_consumed_lexeme_end,
            end: input.len() - blocked_byte_count,
        });

        if !output.is_empty() {
            self.output_sink.handle_chunk(&output);
        }

        self.last_consumed_lexeme_end = 0;
    }

    fn try_produce_token_from_lexeme(&mut self, lexeme: &Lexeme<'_>) {
        let transform_controller = &mut self.transform_controller;
        let output_sink = &mut self.output_sink;
        let lexeme_range = lexeme.raw_range();
        let last_consumed_lexeme_end = self.last_consumed_lexeme_end;
        let mut lexeme_consumed = false;

        self.token_capture.feed(lexeme, &mut |event| match event {
            TokenCapturerEvent::LexemeConsumed => {
                let chunk = lexeme.input().slice(Range {
                    start: last_consumed_lexeme_end,
                    end: lexeme_range.start,
                });

                lexeme_consumed = true;
                output_sink.handle_chunk(&chunk);
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
    fn get_next_parser_output_type(&self) -> NextOutputType {
        if self.token_capture.has_captures() {
            NextOutputType::Lexeme
        } else {
            NextOutputType::TagHint
        }
    }

    fn adjust_capture_flags_for_tag_lexeme(&mut self, lexeme: &Lexeme<'_>) {
        let input = lexeme.input();

        macro_rules! get_flags_from_handler {
            ($handler:expr, $attributes:expr, $self_closing:expr) => {
                $handler(
                    &mut self.transform_controller,
                    ElementModifiersInfo::new(input, Rc::clone($attributes), $self_closing),
                )
                .into()
            };
        }

        let capture_flags = match (
            lexeme.token_outline(),
            self.pending_element_modifiers_info_handler.take(),
        ) {
            // Case 1: we have a start tag and attributes and self closing flags
            // information has been requested in the tag hint handler.
            (
                Some(TokenOutline::StartTag {
                    attributes,
                    self_closing,
                    ..
                }),
                Some(ref mut handler),
            ) => get_flags_from_handler!(handler, attributes, *self_closing),

            // Case 2: we have a start tag for which tag hint handler hasn't been called,
            // because parser uses full state machine at the moment and it doesn't
            // produce hints.
            (
                Some(TokenOutline::StartTag {
                    name,
                    name_hash,
                    attributes,
                    self_closing,
                }),
                None,
            ) => {
                let name_info = TagNameInfo::new(input, *name, *name_hash);

                match self.transform_controller.handle_element_start(name_info) {
                    ElementStartResponse::ContentSettings(settings) => settings.into(),
                    ElementStartResponse::RequestElementModifiersInfo(mut handler) => {
                        get_flags_from_handler!(handler, attributes, *self_closing)
                    }
                }
            }

            // Case 3: we have an end tag for which tag hint handler hasn't been called,
            // because parser uses full state machine at the moment and it doesn't
            // produce hints.
            (Some(TokenOutline::EndTag { name, name_hash }), None) => {
                let name_info = TagNameInfo::new(input, *name, *name_hash);

                self.transform_controller
                    .handle_element_end(name_info)
                    .into()
            }

            // If we got anything else then it's a bug in the implementation
            _ => unreachable!("Impossible combination of lexeme type and pending tag info handler"),
        };

        self.token_capture.set_capture_flags(capture_flags);
    }
}

impl<C, O> LexemeSink for Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    fn handle_tag(&mut self, lexeme: &Lexeme<'_>) -> NextOutputType {
        if self.got_capture_flags_from_tag_hint {
            self.got_capture_flags_from_tag_hint = false;
        } else {
            self.adjust_capture_flags_for_tag_lexeme(lexeme);
        }

        self.try_produce_token_from_lexeme(lexeme);

        // NOTE: we capture tag tokens only for the current lexeme and
        // only if it has been requested in content setting. So, once we've
        // handled lexeme for the tag we should disable tag capturing.
        self.token_capture.stop_capturing_tags();

        self.get_next_parser_output_type()
    }

    #[inline]
    fn handle_non_tag_content(&mut self, lexeme: &Lexeme<'_>) {
        self.try_produce_token_from_lexeme(lexeme);
    }
}

impl<C, O> TagHintSink for Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    fn handle_tag_hint(&mut self, tag_hint: &TagHint<'_>) -> NextOutputType {
        let capture_flags = match *tag_hint {
            TagHint::StartTag(name_info) => {
                match self.transform_controller.handle_element_start(name_info) {
                    ElementStartResponse::ContentSettings(settings) => settings.into(),
                    ElementStartResponse::RequestElementModifiersInfo(handler) => {
                        self.pending_element_modifiers_info_handler = Some(handler);

                        return NextOutputType::Lexeme;
                    }
                }
            }
            TagHint::EndTag(name_info) => self
                .transform_controller
                .handle_element_end(name_info)
                .into(),
        };

        self.token_capture.set_capture_flags(capture_flags);
        self.got_capture_flags_from_tag_hint = true;
        self.get_next_parser_output_type()
    }
}

impl<C, O> ParserOutputSink for Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
}
