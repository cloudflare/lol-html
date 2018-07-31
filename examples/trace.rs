extern crate cool_thing;

use cool_thing::{Token, Tokenizer};
use std::env::args;

fn main() {
    let html = args().nth(1).expect("HTML is not provided.");

    let mut tokenizer = Tokenizer::new(2048, |lex_result| {
        let token: Token = lex_result.as_token();

        println!("------------------");
        println!();
        println!("Token: {:#?}", token);

        if let Some(raw) = lex_result.raw {
            println!("\nRaw: `{}`", unsafe { String::from_utf8_unchecked(raw.to_vec()) });
        }

        println!();
    });

    tokenizer
        .write(html.bytes().collect())
        .expect("Buffer capacity exceeded.");
}
