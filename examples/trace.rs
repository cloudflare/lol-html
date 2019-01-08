use cool_thing::token::Token;
use cool_thing::tokenizer::*;
use cool_thing::transform_stream::{TokenCaptureFlags, TransformController, TransformStream};
use encoding_rs::UTF_8;
use getopts::{Matches, Options};
use std::env::args;

fn parse_options() -> Option<Matches> {
    let mut opts = Options::new();

    opts.optopt(
        "m",
        "text_parsing_mode",
        "Initial text parsing mode",
        "-s (Data state|PLAINTEXT state|RCDATA state|RAWTEXT state|Script data state|CDATA section state)",
    );

    opts.optopt("t", "last_start_tag", "Last start tag name", "-t");
    opts.optopt("c", "chunk_size", "Chunk size", "-c");
    opts.optflag("p", "tag_preview_mode", "Trace in tag preview mode");
    opts.optflag("h", "help", "Show this help");

    let matches = match opts.parse(args().skip(1)) {
        Ok(matches) => {
            if matches.free.is_empty() {
                eprintln!("Missing HTML input");
                None
            } else if matches.opt_present("h") {
                None
            } else {
                Some(matches)
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            None
        }
    };

    if matches.is_none() {
        eprintln!("{}", opts.usage("Usage: trace [options] INPUT"));
    }

    matches
}

struct TraceTransformController {
    tag_preview_mode: bool,
}

impl TraceTransformController {
    pub fn new(tag_preview_mode: bool) -> Self {
        TraceTransformController { tag_preview_mode }
    }
}

impl TransformController for TraceTransformController {
    fn get_initial_token_capture_flags(&self) -> TokenCaptureFlags {
        if self.tag_preview_mode {
            TokenCaptureFlags::empty()
        } else {
            TokenCaptureFlags::all()
        }
    }

    fn get_token_capture_flags_for_tag(&mut self, _: &LexUnit) -> NextOutputType {
        if self.tag_preview_mode {
            NextOutputType::TagPreview
        } else {
            NextOutputType::LexUnit
        }
    }

    fn get_token_capture_flags_for_tag_preview(&mut self, _: &TagPreview) -> NextOutputType {
        if self.tag_preview_mode {
            NextOutputType::TagPreview
        } else {
            NextOutputType::LexUnit
        }
    }

    fn handle_token(&mut self, _: Token) {}
}

fn main() {
    let matches = match parse_options() {
        Some(m) => m,
        None => return,
    };

    let html = matches.free.first().unwrap();
    let tag_preview_mode = matches.opt_present("p");

    let mut transform_stream =
        TransformStream::new(2048, TraceTransformController::new(tag_preview_mode), UTF_8);

    let tokenizer = transform_stream.tokenizer();

    tokenizer.switch_text_parsing_mode(match matches.opt_str("s").as_ref().map(|s| s.as_str()) {
        None => TextParsingMode::Data,
        Some(state) => TextParsingMode::from(state),
    });

    if let Some(ref tag_name) = matches.opt_str("t") {
        tokenizer.set_last_start_tag_name_hash(TagName::get_hash(tag_name));
    }

    let chunks = if let Some(chunk_size) = matches.opt_get("c").unwrap() {
        html.as_bytes().chunks(chunk_size).collect()
    } else {
        vec![html.as_bytes()]
    };

    for chunk in chunks {
        transform_stream.write(chunk).unwrap();
    }

    transform_stream.end().unwrap();
}
