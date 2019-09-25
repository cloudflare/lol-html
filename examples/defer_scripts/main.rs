use cool_thing::{element, HtmlRewriter, Settings};
use std::io;
use std::io::prelude::*;

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Use stdout as an output sink for the rewriter
    let output_sink = |c: &[u8]| {
        stdout.write_all(c).unwrap();
    };

    // Create the rewriter
    let mut rewriter = HtmlRewriter::try_new(
        Settings {
            element_content_handlers: vec![element!(
                "script[src]:not([async]):not([defer])",
                |el| {
                    el.set_attribute("defer", "").unwrap();
                    Ok(())
                }
            )],
            ..Settings::default()
        },
        output_sink,
    )
    .unwrap();

    // Feed chunks from the stdin to the rewriter
    loop {
        let mut stdin = stdin.lock();
        let buffer = stdin.fill_buf().unwrap();
        let len = buffer.len();

        if len > 0 {
            rewriter.write(buffer).unwrap();
            stdin.consume(len);
        } else {
            rewriter.end().unwrap();
            break;
        }
    }
}
