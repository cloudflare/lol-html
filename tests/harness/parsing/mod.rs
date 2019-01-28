mod chunked_input;

use cool_thing::parser::{Lexeme, NextOutputType, TagHint, TextType};
use cool_thing::token::{Token, TokenCaptureFlags};
use cool_thing::transform_stream::{Output, TransformController, TransformStream};
use failure::Error;

pub use self::chunked_input::ChunkedInput;

struct TestTransformController<'h> {
    token_handler: Box<dyn FnMut(Token<'_>) -> Output<'_> + 'h>,
    capture_flags: TokenCaptureFlags,
}

impl<'h> TestTransformController<'h> {
    pub fn new(
        token_handler: Box<dyn FnMut(Token<'_>) -> Output<'_> + 'h>,
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

    fn handle_token<'t>(&mut self, token: Token<'t>) -> Output<'t> {
        (self.token_handler)(token)
    }
}

pub fn parse<'h>(
    input: &ChunkedInput,
    capture_flags: TokenCaptureFlags,
    initial_text_type: TextType,
    last_start_tag_name_hash: Option<u64>,
    token_handler: Box<dyn FnMut(Token<'_>) -> Output<'_> + 'h>,
) -> Result<String, Error> {
    let mut output = Vec::new();

    let encoding = input
        .encoding()
        .expect("Input should be initialized before parsing");

    let transform_controller = TestTransformController::new(token_handler, capture_flags);

    let mut transform_stream = TransformStream::new(
        transform_controller,
        |chunk: &[u8]| output.extend_from_slice(chunk),
        2048,
        encoding,
    );

    let parser = transform_stream.parser();

    parser.set_last_start_tag_name_hash(last_start_tag_name_hash);
    parser.switch_text_type(initial_text_type);

    for chunk in input.chunks() {
        transform_stream.write(chunk)?;
    }

    transform_stream.end()?;

    Ok(encoding.decode_without_bom_handling(&output).0.to_string())
}
