mod chunked_input;

use cool_thing::parser::{Lexeme, NextOutputType, TagHint, TextType};
use cool_thing::token::{Token, TokenCaptureFlags};
use cool_thing::transform_stream::TransformController;
use failure::Error;

pub use self::chunked_input::ChunkedInput;

struct TestTransformController<'h> {
    token_handler: Box<dyn FnMut(Token<'_>) + 'h>,
    capture_flags: TokenCaptureFlags,
}

impl<'h> TestTransformController<'h> {
    pub fn new(
        token_handler: Box<dyn FnMut(Token<'_>) + 'h>,
        capture_flags: TokenCaptureFlags,
    ) -> Self {
        TestTransformController {
            token_handler,
            capture_flags,
        }
    }
}

impl TransformController for TestTransformController<'_> {
    fn get_initial_token_capture_flags(&self) -> TokenCaptureFlags {
        self.capture_flags
    }

    fn get_token_capture_flags_for_tag(&mut self, _: &Lexeme) -> NextOutputType {
        NextOutputType::Lexeme
    }

    fn get_token_capture_flags_for_tag_hint(&mut self, _: &TagHint) -> NextOutputType {
        NextOutputType::Lexeme
    }

    fn handle_token(&mut self, token: Token<'_>) {
        (self.token_handler)(token);
    }
}

pub fn parse<'h>(
    input: &ChunkedInput,
    capture_flags: TokenCaptureFlags,
    initial_text_type: TextType,
    last_start_tag_name_hash: Option<u64>,
    token_handler: Box<dyn FnMut(Token<'_>) + 'h>,
) -> Result<(), Error> {
    input.parse(
        TestTransformController::new(token_handler, capture_flags),
        initial_text_type,
        last_start_tag_name_hash,
    )
}
