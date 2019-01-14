use crate::token::{Token, TokenCapture, TokenCaptureFlags, TokenCaptureResult};
use crate::tokenizer::{
    LexUnit, LexUnitSink, NextOutputType, OutputSink as TokenizerOutputSink, TagPreview,
    TagPreviewSink,
};
use encoding_rs::Encoding;
use std::cell::RefCell;

// TODO OutputSink
// handle_bailout
pub trait TransformController {
    fn get_initial_token_capture_flags(&self) -> TokenCaptureFlags;
    fn get_token_capture_flags_for_tag(&mut self, tag_lex_unit: &LexUnit) -> NextOutputType;

    fn get_token_capture_flags_for_tag_preview(
        &mut self,
        tag_preview: &TagPreview,
    ) -> NextOutputType;

    fn handle_token(&mut self, token: Token);
}

pub struct Writer<C: TransformController> {
    transform_controller: RefCell<C>,
    token_capture: TokenCapture,
}

impl<C: TransformController> Writer<C> {
    pub fn new(transform_controller: C, encoding: &'static Encoding) -> Self {
        let initial_capture_flags = transform_controller.get_initial_token_capture_flags();

        Writer {
            transform_controller: RefCell::new(transform_controller),
            token_capture: TokenCapture::new(initial_capture_flags, encoding),
        }
    }

    fn handle_lex_unit(&mut self, lex_unit: &LexUnit<'_>) {
        let mut transform_controller = self.transform_controller.borrow_mut();

        self.token_capture.feed(lex_unit, &mut |res| {
            if let TokenCaptureResult::Captured(token) = res {
                transform_controller.handle_token(token);
            }
        });
    }
}

impl<C: TransformController> LexUnitSink for Writer<C> {
    #[inline]
    fn handle_tag(&mut self, lex_unit: &LexUnit<'_>) -> NextOutputType {
        let next_output_type = self
            .transform_controller
            .borrow_mut()
            .get_token_capture_flags_for_tag(lex_unit);

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
        self.transform_controller
            .borrow_mut()
            .get_token_capture_flags_for_tag_preview(tag_preview)
    }
}

impl<C: TransformController> TokenizerOutputSink for Writer<C> {}
