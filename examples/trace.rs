extern crate cool_thing;

use cool_thing::*;
use std::env::args;

fn main() {
    let html = args().nth(1).expect("HTML is not provided.");

    let mut tokenizer = Tokenizer::new(2048, |token| {
        println!("{:#?}", token);
    });

    tokenizer
        .write(html.bytes().collect())
        .expect("Buffer capacity exceeded.");
}
