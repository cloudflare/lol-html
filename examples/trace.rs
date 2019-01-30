use cool_thing::parser::*;
use cool_thing::token::{Token, TokenCaptureFlags};
use cool_thing::transform_stream::{TransformController, TransformStream};
use encoding_rs::UTF_8;
use getopts::{Matches, Options};
use std::env::args;

fn parse_options() -> Option<Matches> {
    let mut opts = Options::new();

    opts.optopt(
        "t",
        "text_type",
        "Initial text type",
        "-t (Data state|PLAINTEXT state|RCDATA state|RAWTEXT state|Script data state|CDATA section state)",
    );

    opts.optopt("l", "last_start_tag", "Last start tag name", "-l");
    opts.optopt("c", "chunk_size", "Chunk size", "-c");
    opts.optflag("p", "tag_hint_mode", "Trace in tag preview mode");
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
    tag_hint_mode: bool,
}

impl TraceTransformController {
    pub fn new(tag_hint_mode: bool) -> Self {
        TraceTransformController { tag_hint_mode }
    }
}

impl TransformController for TraceTransformController {
    fn get_initial_token_capture_flags(&self) -> TokenCaptureFlags {
        if self.tag_hint_mode {
            TokenCaptureFlags::empty()
        } else {
            TokenCaptureFlags::all()
        }
    }

    fn get_token_capture_flags_for_tag(&mut self, _: &Lexeme) -> NextOutputType {
        if self.tag_hint_mode {
            NextOutputType::TagHint
        } else {
            NextOutputType::Lexeme
        }
    }

    fn get_token_capture_flags_for_tag_hint(&mut self, _: &TagHint) -> NextOutputType {
        if self.tag_hint_mode {
            NextOutputType::TagHint
        } else {
            NextOutputType::Lexeme
        }
    }

    fn handle_token(&mut self, _: &mut Token<'_>) {}
}

fn main() {
    let matches = match parse_options() {
        Some(m) => m,
        None => return,
    };

    let html = matches.free.first().unwrap();
    let tag_hint_mode = matches.opt_present("p");

    let mut transform_stream = TransformStream::new(
        TraceTransformController::new(tag_hint_mode),
        |_: &[u8]| {},
        2048,
        UTF_8,
    );

    let parser = transform_stream.parser();

    parser.switch_text_type(match matches.opt_str("t").as_ref().map(|s| s.as_str()) {
        None => TextType::Data,
        Some(state) => TextType::from(state),
    });

    if let Some(ref tag_name) = matches.opt_str("l") {
        parser.set_last_start_tag_name_hash(TagName::get_hash(tag_name));
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
