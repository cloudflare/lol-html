use cool_thing::errors::*;
use cool_thing::*;
use encoding_rs::UTF_8;

struct BenchTransformController {
    capture_flags: TokenCaptureFlags,
}

impl BenchTransformController {
    pub fn new(capture_flags: TokenCaptureFlags) -> Self {
        BenchTransformController { capture_flags }
    }
}

impl TransformController for BenchTransformController {
    fn initial_capture_flags(&self) -> TokenCaptureFlags {
        self.capture_flags
    }

    fn handle_start_tag(&mut self, name: LocalName, ns: Namespace) -> StartTagHandlingResult<Self> {
        black_box(name);
        black_box(ns);

        Ok(self.capture_flags)
    }

    fn handle_end_tag(&mut self, name: LocalName) -> TokenCaptureFlags {
        black_box(name);

        self.capture_flags
    }

    fn handle_token(&mut self, token: &mut Token) -> Result<(), RewritingError> {
        black_box(token);

        Ok(())
    }

    fn should_emit_content(&self) -> bool {
        true
    }
}

fn create_runner(capture_flags: TokenCaptureFlags) -> impl FnMut(&mut Bencher, &Vec<Vec<u8>>) {
    move |b, i: &Vec<Vec<u8>>| {
        b.iter(|| {
            let mut transform_stream = TransformStream::new(TransformStreamSettings {
                transform_controller: BenchTransformController::new(capture_flags),
                output_sink: |c: &[u8]| {
                    black_box(c);
                },
                preallocated_parsing_buffer_size: 2048,
                memory_limiter: MemoryLimiter::new_shared(std::usize::MAX),
                encoding: UTF_8,
                strict: true,
            });

            for chunk in i {
                transform_stream.write(chunk).unwrap();
            }

            transform_stream.end().unwrap();
        })
    }
}

define_group!(
    "Parsing",
    [
        ("Tag scanner", create_runner(TokenCaptureFlags::empty())),
        (
            "Lexer",
            // NOTE: this switches parser to the lexer mode and doesn't
            // trigger token production for anything, except doctype. So,
            // we can get relatively fair comparison.
            create_runner(TokenCaptureFlags::DOCTYPES)
        ),
        (
            "Text rewritable unit parsing and decoding",
            // NOTE: this is the biggest bottleneck part of the parser and rewriter.
            // It's not guaranteed that chunks that come over the wire contain decodable
            // sequence of bytes for the given character encoding. So, if there is a text
            // handler in the selector matching scope, we need to slice and decode all
            // incoming chunks to produce correct text chunk rewritable units.
            create_runner(TokenCaptureFlags::TEXT)
        )
    ]
);
