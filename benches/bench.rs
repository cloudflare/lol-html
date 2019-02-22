#[macro_use]
extern crate criterion;

use cool_thing::transform_stream::{
    ContentSettingsOnElementEnd, ContentSettingsOnElementStart, DocumentLevelContentSettings,
};
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

#[derive(Copy, Clone)]
pub struct CoolThingContentSettings {
    pub document_level: DocumentLevelContentSettings,
    pub on_element_start: ContentSettingsOnElementStart,
    pub on_element_end: ContentSettingsOnElementEnd,
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
    content_settings: CoolThingContentSettings,
) -> impl FnMut(&mut Bencher, &Input) {
    move |b, i: &Input| {
        use cool_thing::parser::TagNameInfo;
        use cool_thing::token::Token;
        use cool_thing::transform_stream::{
            ElementStartResponse, TransformController, TransformStream,
        };

        struct BenchTransformController {
            content_settings: CoolThingContentSettings,
        }

        impl BenchTransformController {
            pub fn new(content_settings: CoolThingContentSettings) -> Self {
                BenchTransformController { content_settings }
            }
        }

        impl TransformController for BenchTransformController {
            fn document_level_content_settings(&self) -> DocumentLevelContentSettings {
                self.content_settings.document_level
            }

            fn handle_element_start(
                &mut self,
                name_info: &TagNameInfo<'_>,
            ) -> ElementStartResponse<Self> {
                black_box(name_info);
                ElementStartResponse::ContentSettings(self.content_settings.on_element_start)
            }

            fn handle_element_end(
                &mut self,
                name_info: &TagNameInfo<'_>,
            ) -> ContentSettingsOnElementEnd {
                black_box(name_info);
                self.content_settings.on_element_end
            }

            fn handle_token(&mut self, token: &mut Token<'_>) {
                black_box(token);
            }
        }

        b.iter(|| {
            let mut transform_stream = TransformStream::new(
                BenchTransformController::new(content_settings),
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
            cool_thing_tokenizer_bench(CoolThingContentSettings {
                document_level: DocumentLevelContentSettings::empty(),
                on_element_start: ContentSettingsOnElementStart::empty(),
                on_element_end: ContentSettingsOnElementEnd::empty(),
            }),
            inputs,
        )
        .with_function(
            "cool_thing - Capture everything except text",
            cool_thing_tokenizer_bench(CoolThingContentSettings {
                document_level: DocumentLevelContentSettings::CAPTURE_COMMENTS
                    | DocumentLevelContentSettings::CAPTURE_DOCTYPES,
                on_element_start: ContentSettingsOnElementStart::CAPTURE_START_TAG_FOR_ELEMENT,
                on_element_end: ContentSettingsOnElementEnd::CAPTURE_END_TAG_FOR_ELEMENT,
            }),
        )
        .with_function(
            "cool_thing - Capture everything",
            cool_thing_tokenizer_bench(CoolThingContentSettings {
                document_level: DocumentLevelContentSettings::all(),
                on_element_start: ContentSettingsOnElementStart::all(),
                on_element_end: ContentSettingsOnElementEnd::all(),
            }),
        )
        .with_function("lazyhtml", lazyhtml_tokenizer_bench())
        .with_function("html5ever", html5ever_tokenizer_bench())
        .throughput(|i| Throughput::Bytes(i.length as u32)),
    );
}

criterion_group!(benches, tokenization_benchmark);
criterion_main!(benches);
