use lol_html::{HtmlRewriter, Settings, element};
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
    let mut rewriter = HtmlRewriter::new(
        Settings::new().append_element_content_handler(element!(
            "script[src]:not([async]):not([defer])",
            |el| {
                el.set_attribute("defer", "").unwrap();
                Ok(())
            }
        )),
        output_sink,
    );

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
