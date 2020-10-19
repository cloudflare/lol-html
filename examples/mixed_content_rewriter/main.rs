use lol_html::html_content::Element;
use lol_html::{element, HtmlRewriter, Settings};
use std::io;
use std::io::prelude::*;

fn rewrite_url_in_attr(el: &mut Element, attr_name: &str) {
    let attr = el
        .get_attribute(attr_name)
        .unwrap()
        .replace("http://", "https://");

    el.set_attribute(attr_name, &attr).unwrap();
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Use stdout as an output sink for the rewriter
    let output_sink = |c: &[u8]| {
        stdout.write_all(c).unwrap();
    };

    // Create the rewriter
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("a[href], link[rel=stylesheet][href]", |el| {
                    rewrite_url_in_attr(el, "href");
                    Ok(())
                }),
                element!(
                    "script[src], iframe[src], img[src], audio[src], video[src]",
                    |el| {
                        rewrite_url_in_attr(el, "src");
                        Ok(())
                    }
                ),
            ],
            ..Settings::default()
        },
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
