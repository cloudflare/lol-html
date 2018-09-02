extern crate cool_thing;
extern crate getopts;

use cool_thing::tokenizer::*;
use getopts::{Matches, Options};
use std::env::args;

fn parse_options() -> Option<Matches> {
    let mut opts = Options::new();

    opts.optopt(
        "s",
        "state",
        "Initial state",
        "-s (Data state|PLAINTEXT state|RCDATA state|RAWTEXT state|Script data state|CDATA section state)",
    );

    opts.optopt("t", "last_start_tag", "Last start tag name", "-l");

    opts.optflag("h", "help", "Show this help");

    let matches = match opts.parse(args().skip(1)) {
        Ok(matches) => if matches.free.is_empty() {
            eprintln!("Missing HTML input");
            None
        } else if matches.opt_present("h") {
            None
        } else {
            Some(matches)
        },
        Err(e) => {
            eprintln!("{}", e);
            None
        }
    };

    if let None = matches {
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

    let mut tokenizer = Tokenizer::new(2048, |lex_result: LexResult| {
        println!("------------------");

        if let Some(token) = lex_result.as_token() {
            println!("Shallow token: {:#?}", lex_result.shallow_token.unwrap());
            println!();
            println!("Token: {:#?}", token);
        }

        if let Some(raw) = lex_result.raw {
            println!("\nRaw: `{}`", unsafe {
                String::from_utf8_unchecked(raw.to_vec())
            });
        }

        println!();
    });

    tokenizer.set_state(match matches.opt_str("s").as_ref().map(|s| s.as_str()) {
        None => Tokenizer::data_state,
        Some(state) => TextParsingMode::from(state).into(),
    });

    if let Some(ref tag_name) = matches.opt_str("t") {
        tokenizer.set_last_start_tag_name_hash(get_tag_name_hash(tag_name));
    }

    tokenizer
        .write(html.bytes().collect())
        .expect("Buffer capacity exceeded.");
}
