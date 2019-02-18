use super::TransformController;
use crate::base::{Chunk, Range};
use crate::parser::{Lexeme, LexemeSink, NextOutputType, ParserOutputSink, TagHint, TagHintSink};
use crate::token::{Serialize, TokenCapture, TokenCaptureEvent};
use encoding_rs::Encoding;
use std::cell::RefCell;

pub trait OutputSink {
    fn handle_chunk(&mut self, chunk: &[u8]);
}

impl<F: FnMut(&[u8])> OutputSink for F {
    fn handle_chunk(&mut self, chunk: &[u8]) {
        self(chunk);
    }
}

// handle_element_start(tag_name_info)
// -> ElementStartResponse {
//  GetFullInfo(Fn -> StartTagResponse) //attrs, self closing, ns
//  ContentSettingsOnElementStart
// }

// handle_element_end(tag_name_info)
// -> ContentSettingsOnElementEnd
// TrasformController - separate file

pub struct Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    transform_controller: RefCell<C>,
    output_sink: RefCell<O>,
    last_consumed_lexeme_end: usize,
    token_capture: TokenCapture,
}

impl<C, O> Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    pub fn new(transform_controller: C, output_sink: O, encoding: &'static Encoding) -> Self {
        let initial_capture_flags = transform_controller.get_initial_token_capture_flags();

        Dispatcher {
            transform_controller: RefCell::new(transform_controller),
            output_sink: RefCell::new(output_sink),
            last_consumed_lexeme_end: 0,
            token_capture: TokenCapture::new(initial_capture_flags, encoding),
        }
    }

    pub fn flush_remaining_input(&mut self, input: &Chunk<'_>, blocked_byte_count: usize) {
        let output = input.slice(Range {
            start: self.last_consumed_lexeme_end,
            end: input.len() - blocked_byte_count,
        });

        if !output.is_empty() {
            self.output_sink.borrow_mut().handle_chunk(&output);
        }

        self.last_consumed_lexeme_end = 0;
    }

    fn handle_lexeme(&mut self, lexeme: &Lexeme<'_>) {
        let mut transform_controller = self.transform_controller.borrow_mut();
        let mut output_sink = self.output_sink.borrow_mut();
        let mut lexeme_consumed = false;
        let lexeme_range = lexeme.raw_range();
        let last_consumed_lexeme_end = self.last_consumed_lexeme_end;

        self.token_capture.feed(lexeme, &mut |event| match event {
            TokenCaptureEvent::LexemeConsumed => {
                let chunk = lexeme.input().slice(Range {
                    start: last_consumed_lexeme_end,
                    end: lexeme_range.start,
                });

                lexeme_consumed = true;
                output_sink.handle_chunk(&chunk);
            }
            TokenCaptureEvent::TokenProduced(mut token) => {
                trace!(@output token);

                transform_controller.handle_token(&mut token);
                token.to_bytes(&mut |c| output_sink.handle_chunk(c));
            }
        });

        if lexeme_consumed {
            self.last_consumed_lexeme_end = lexeme_range.end;
        }
    }
}

impl<C, O> LexemeSink for Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    #[inline]
    fn handle_tag(&mut self, lexeme: &Lexeme<'_>) -> NextOutputType {
        let next_output_type = self
            .transform_controller
            .borrow_mut()
            .get_token_capture_flags_for_tag(lexeme);

        self.handle_lexeme(lexeme);

        next_output_type
    }

    #[inline]
    fn handle_non_tag_content(&mut self, lexeme: &Lexeme<'_>) {
        self.handle_lexeme(lexeme);
    }
}

impl<C, O> TagHintSink for Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
    #[inline]
    fn handle_tag_hint(&mut self, tag_hint: &TagHint<'_>) -> NextOutputType {
        self.transform_controller
            .borrow_mut()
            .get_token_capture_flags_for_tag_hint(tag_hint)
    }
}

impl<C, O> ParserOutputSink for Dispatcher<C, O>
where
    C: TransformController,
    O: OutputSink,
{
}
