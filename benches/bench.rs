#[macro_use]
extern crate criterion;

use cool_thing::token::TokenCaptureFlags;
use criterion::{black_box, Bencher, Criterion, ParameterizedBenchmark, Throughput};
use encoding_rs::UTF_8;
use glob::glob;
use std::fmt::{self, Debug};
use std::fs::File;
use std::io::Read;

const CHUNK_SIZE: usize = 1024;

struct Input {
    pub name: String,
    pub length: usize,
    pub chunks: Vec<String>,
}

impl Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn get_inputs() -> Vec<Input> {
    glob("benches/data/*.html")
        .unwrap()
        .map(|path| {
            let mut data = String::new();
            let path = path.unwrap();

            File::open(&path)
                .unwrap()
                .read_to_string(&mut data)
                .unwrap();

            Input {
                name: path.file_name().unwrap().to_string_lossy().to_string(),
                length: data.as_bytes().len(),
                chunks: data
                    .into_bytes()
                    .chunks(CHUNK_SIZE)
                    .map(|c| unsafe { String::from_utf8_unchecked(c.to_vec()) })
                    .collect(),
            }
        })
        .collect()
}

fn cool_thing_tokenizer_bench(
    capture_flags: TokenCaptureFlags,
) -> impl FnMut(&mut Bencher, &Input) {
    move |b, i: &Input| {
        use cool_thing::parser::{Lexeme, NextOutputType, TagHint};
        use cool_thing::token::Token;
        use cool_thing::transform_stream::{TransformController, TransformStream};

        struct BenchTransformController {
            capture_flags: TokenCaptureFlags,
        }

        impl BenchTransformController {
            pub fn new(capture_flags: TokenCaptureFlags) -> Self {
                BenchTransformController { capture_flags }
            }
        }

        impl TransformController for BenchTransformController {
            fn get_initial_token_capture_flags(&self) -> TokenCaptureFlags {
                self.capture_flags
            }

            fn get_token_capture_flags_for_tag(&mut self, tag_lexeme: &Lexeme) -> NextOutputType {
                black_box(tag_lexeme);

                if self.capture_flags.is_empty() {
                    NextOutputType::TagHint
                } else {
                    NextOutputType::Lexeme
                }
            }

            fn get_token_capture_flags_for_tag_hint(
                &mut self,
                tag_hint: &TagHint,
            ) -> NextOutputType {
                black_box(tag_hint);

                if self.capture_flags.is_empty() {
                    NextOutputType::TagHint
                } else {
                    NextOutputType::Lexeme
                }
            }

            fn handle_token(&mut self, token: &mut Token<'_>) {
                black_box(token);
            }
        }

        b.iter(|| {
            let mut transform_stream = TransformStream::new(
                BenchTransformController::new(capture_flags),
                |_: &[u8]| {},
                2048,
                UTF_8,
            );

            for chunk in &i.chunks {
                transform_stream.write(chunk.as_bytes()).unwrap();
            }

            transform_stream.end().unwrap();
        })
    }
}

fn lazyhtml_tokenizer_bench() -> impl FnMut(&mut Bencher, &Input) {
    |b, i: &Input| {
        use lazyhtml::*;
        use std::os::raw::c_void;
        use std::ptr::null_mut;

        unsafe extern "C" fn handle_token(token: *mut lhtml_token_t, _state: *mut c_void) {
            black_box(*token);
        }

        b.iter(|| {
            let mut handler = lhtml_token_handler_t {
                callback: Some(handle_token),
                next: null_mut(),
            };

            let mut tokenizer = lazyhtml::Tokenizer::new(2048, 256);

            handler.inject_into(&mut tokenizer);

            for chunk in &i.chunks {
                tokenizer.feed(chunk).unwrap();
            }

            tokenizer.end().unwrap();
        })
    }
}

fn html5ever_tokenizer_bench() -> impl FnMut(&mut Bencher, &Input) {
    |b, i: &Input| {
        use html5ever::tendril::StrTendril;
        use html5ever::tokenizer::{
            BufferQueue, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
            TokenizerResult,
        };

        struct Sink;

        impl TokenSink for Sink {
            type Handle = ();

            fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
                black_box(token);
                TokenSinkResult::Continue
            }
        }

        b.iter(|| {
            let mut tokenizer = Tokenizer::new(Sink, TokenizerOpts::default());
            let mut queue = BufferQueue::new();

            for chunk in &i.chunks {
                queue.push_back(StrTendril::from_slice(chunk));

                while let TokenizerResult::Script(_) = tokenizer.feed(&mut queue) {
                    // ignore script markers
                }
            }

            tokenizer.end();
        })
    }
}

fn tokenization_benchmark(c: &mut Criterion) {
    let inputs = get_inputs();

    c.bench(
        "Tokenizer",
        ParameterizedBenchmark::new(
            "cool_thing - Fast scan",
            cool_thing_tokenizer_bench(TokenCaptureFlags::empty()),
            inputs,
        )
        .with_function(
            "cool_thing - Capture everything except text",
            cool_thing_tokenizer_bench(
                TokenCaptureFlags::DOCTYPES
                    | TokenCaptureFlags::START_TAGS
                    | TokenCaptureFlags::END_TAGS
                    | TokenCaptureFlags::COMMENTS,
            ),
        )
        .with_function(
            "cool_thing - Capture everything",
            cool_thing_tokenizer_bench(TokenCaptureFlags::all()),
        )
        .with_function("lazyhtml", lazyhtml_tokenizer_bench())
        .with_function("html5ever", html5ever_tokenizer_bench())
        .throughput(|i| Throughput::Bytes(i.length as u32)),
    );
}

criterion_group!(benches, tokenization_benchmark);
criterion_main!(benches);
