#[macro_use]
extern crate criterion;

extern crate cool_thing;
extern crate glob;
extern crate html5ever;
extern crate lazyhtml;

use criterion::{black_box, Bencher, Criterion, ParameterizedBenchmark, Throughput};
use glob::glob;
use std::fmt::{self, Debug};
use std::fs::File;
use std::io::Read;

const CHUNK_SIZE: usize = 1024;

struct Input {
    pub name: String,
    pub chunks: Vec<String>,
}

impl Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
                chunks: data
                    .into_bytes()
                    .chunks(CHUNK_SIZE)
                    .map(|c| unsafe { String::from_utf8_unchecked(c.to_vec()) })
                    .collect(),
            }
        }).collect()
}

fn cool_thing_tokenizer_bench() -> impl FnMut(&mut Bencher, &Input) {
    |b, i: &Input| {
        use cool_thing::tokenizer::{LexUnit, TagPreview};
        use cool_thing::transform_stream::TransformStream;

        b.iter(|| {
            let mut transform_stream = TransformStream::new(
                2048,
                |lex_unit: &LexUnit| {
                    black_box(lex_unit);
                },
                |_tag_preview: &TagPreview| {},
            );

            transform_stream.get_tokenizer().tag_preview_mode(false);

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
        ParameterizedBenchmark::new("cool_thing", cool_thing_tokenizer_bench(), inputs)
            .with_function("lazyhtml", lazyhtml_tokenizer_bench())
            .with_function("html5ever", html5ever_tokenizer_bench())
            .throughput(|i| Throughput::Bytes(i.chunks.iter().fold(0, |s, c| s + c.len() as u32))),
    );
}

criterion_group!(benches, tokenization_benchmark);
criterion_main!(benches);
