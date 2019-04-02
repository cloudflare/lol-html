use super::{Input, Output};
use cool_thing::*;
use failure::Error;

type TokenHandler<'h> = Box<dyn FnMut(&mut Token<'_>) + 'h>;

struct TestTransformController<'h> {
    token_handler: TokenHandler<'h>,
    capture_flags: TokenCaptureFlags,
}

impl<'h> TestTransformController<'h> {
    pub fn new(token_handler: TokenHandler<'h>, capture_flags: TokenCaptureFlags) -> Self {
        TestTransformController {
            token_handler,
            capture_flags,
        }
    }
}

impl TransformController for TestTransformController<'_> {
    fn initial_capture_flags(&self) -> TokenCaptureFlags {
        self.capture_flags
    }
    fn handle_element_start(&mut self, _: &LocalName<'_>) -> ElementStartResponse<Self> {
        ElementStartResponse::CaptureFlags(self.capture_flags)
    }

    fn handle_element_end(&mut self, _: &LocalName<'_>) -> TokenCaptureFlags {
        self.capture_flags
    }

    fn handle_token(&mut self, token: &mut Token<'_>) {
        (self.token_handler)(token)
    }
}

pub fn parse(
    input: &Input,
    capture_flags: TokenCaptureFlags,
    initial_text_type: TextType,
    last_start_tag_name_hash: LocalNameHash,
    token_handler: TokenHandler<'_>,
) -> Result<String, Error> {
    let encoding = input
        .encoding()
        .expect("Input should be initialized before parsing");

    let mut output = Output::new(encoding);

    let transform_controller = TestTransformController::new(token_handler, capture_flags);

    let mut transform_stream = TransformStream::new(
        transform_controller,
        |chunk: &[u8]| output.push(chunk),
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

    Ok(output.into())
}

macro_rules! parse_token {
    ($input:expr, $encoding:expr, $TokenType:ident, $callback:expr) => {{
        use crate::harness::{parse, Input};
        use cool_thing::{LocalNameHash, TextType, Token, TokenCaptureFlags};

        let mut input: Input = String::from($input).into();
        let mut emitted = false;

        input.init($encoding, true).unwrap();

        parse(
            &input,
            TokenCaptureFlags::all(),
            TextType::Data,
            LocalNameHash::default(),
            Box::new(move |t| match t {
                Token::$TokenType(t) => {
                    // NOTE: we always have two text chunks:
                    // one with the actual text and the second is emitted
                    // on EOF to signify the end of the text node.
                    // We need to invoke callback only for the first one.
                    if !emitted {
                        $callback(t);
                        emitted = true;
                    }
                }
                _ => unreachable!("Input should contain only tokens of the requested type"),
            }),
        )
        .unwrap();
    }};
}
