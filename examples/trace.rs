extern crate cool_thing;

use cool_thing::{Token, Tokenizer};
use std::env::args;

fn main() {
    let html = args().nth(1).expect("HTML is not provided.");

    let mut tokenizer = Tokenizer::new(2048, |lex_result| {
        let token: Token = lex_result.into();

        println!("{:#?}", token);
    });

    tokenizer
        .write(html.bytes().collect())
        .expect("Buffer capacity exceeded.");
}
