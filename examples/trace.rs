extern crate cool_thing;
extern crate getopts;

use cool_thing::*;
use getopts::{Matches, Options};
use std::env::args;

fn parse_options() -> Option<Matches> {
    let mut opts = Options::new();

    opts.optopt(
        "s",
        "state",
        "Initial state",
        "-s (Data|PlainText|RCData|RawText|ScriptData|CDataSection)",
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

    let mut tokenizer = Tokenizer::new(2048, |lex_result| {
        let token: Token = lex_result.as_token();

        println!("------------------");
        println!("Shallow token: {:#?}", lex_result.shallow_token);
        println!();
        println!("Token: {:#?}", token);

        if let Some(raw) = lex_result.raw {
            println!("\nRaw: `{}`", unsafe {
                String::from_utf8_unchecked(raw.to_vec())
            });
        }

        println!();
    });

    tokenizer.set_state(match matches.opt_str("s").as_ref().map(|s| s.as_str()) {
        None | Some("Data") => Tokenizer::data_state,
        Some("PlainText") => Tokenizer::plaintext_state,
        Some("RCData") => Tokenizer::rcdata_state,
        Some("RawText") => Tokenizer::rawtext_state,
        Some("ScriptData") => Tokenizer::script_data_state,
        Some("CDataSection") => Tokenizer::cdata_section_state,
        _ => {
            eprintln!("Unknown state provided");
            return;
        }
    });

    if let Some(ref tag_name) = matches.opt_str("t") {
        tokenizer.set_last_start_tag_name_hash(get_tag_name_hash(tag_name));
    }

    tokenizer
        .write(html.bytes().collect())
        .expect("Buffer capacity exceeded.");
}
