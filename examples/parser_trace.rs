use cool_thing::*;
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
    opts.optflag("H", "tag_hint_mode", "Trace in tag hint mode");
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
    capture_flags: TokenCaptureFlags,
}

impl TraceTransformController {
    pub fn new(tag_hint_mode: bool) -> Self {
        TraceTransformController {
            capture_flags: if tag_hint_mode {
                TokenCaptureFlags::empty()
            } else {
                TokenCaptureFlags::all()
            },
        }
    }
}

impl TransformController for TraceTransformController {
    fn initial_capture_flags(&self) -> TokenCaptureFlags {
        self.capture_flags
    }

    fn handle_start_tag(&mut self, _: LocalName, _: Namespace) -> StartTagHandlingResult<Self> {
        Ok(self.capture_flags)
    }

    fn handle_end_tag(&mut self, _: LocalName) -> TokenCaptureFlags {
        self.capture_flags
    }

    fn handle_token(&mut self, _: &mut Token) -> ConsequentContentDirective {
        ConsequentContentDirective::None
    }
}

fn main() {
    let matches = match parse_options() {
        Some(m) => m,
        None => return,
    };

    let html = matches.free.first().unwrap();
    let tag_hint_mode = matches.opt_present("H");

    let mut transform_stream = TransformStream::new(
        TraceTransformController::new(tag_hint_mode),
        |_: &[u8]| {},
        2048,
        UTF_8,
    );

    let parser = transform_stream.parser();

    parser.switch_text_type(match matches.opt_str("t").as_ref().map(String::as_str) {
        None => TextType::Data,
        Some(state) => TextType::from(state),
    });

    if let Some(ref tag_name) = matches.opt_str("l") {
        parser.set_last_start_tag_name_hash(tag_name.as_str().into());
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
