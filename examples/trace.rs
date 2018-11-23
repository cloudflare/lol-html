extern crate cool_thing;
extern crate getopts;

use cool_thing::tokenizer::*;
use cool_thing::transform_stream::TransformStream;
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
    opts.optflag(
        "s",
        "tag_scan_mode",
        "Trace in tag scan mode: emit tag previews and corresponding lex units",
    );
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

fn main() {
    let matches = match parse_options() {
        Some(m) => m,
        None => return,
    };

    let html = matches.free.first().unwrap();
    let tag_scan_mode = matches.opt_present("s");

    let mut transform_stream = TransformStream::new(
        2048,
        |_lex_unit: &LexUnit| {},
        |_lex_unit: &LexUnit| {
            if tag_scan_mode {
                NextOutputType::TagPreview
            } else {
                NextOutputType::LexUnit
            }
        },
        |_tag_preview: &TagPreview| {
            if tag_scan_mode {
                NextOutputType::LexUnit
            } else {
                NextOutputType::TagPreview
            }
        },
    );

    {
        let tokenizer = transform_stream.get_tokenizer();

        tokenizer.set_next_output_type(if tag_scan_mode || matches.opt_present("p") {
            NextOutputType::TagPreview
        } else {
            NextOutputType::LexUnit
        });

        tokenizer.switch_text_parsing_mode(
            match matches.opt_str("s").as_ref().map(|s| s.as_str()) {
                None => TextParsingMode::Data,
                Some(state) => TextParsingMode::from(state),
            },
        );

        if let Some(ref tag_name) = matches.opt_str("t") {
            tokenizer.set_last_start_tag_name_hash(TagName::get_hash(tag_name));
        }
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
