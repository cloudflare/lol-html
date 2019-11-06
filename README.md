![Banner](media/banner.png)

*Cool Thing is a streaming HTML rewriter/parser with CSS-selector based API.*

It is designed to provide low output latency, quickly handle big amounts of data and operate in
environments with limited memory resources.

The crate serves as a back-end for the HTML rewriting functionality of
[Cloudflare Workers](https://www.cloudflare.com/en-gb/products/cloudflare-workers/), but can be used
as a standalone library with the convenient API for a wide variety of HTML rewriting/analyzis tasks.

# Documentation

https://docs.rs/cool-thing

# Example

Rewrite insecure hyperlinks:

```rust
use cool_thing::{element, HtmlRewriter, Settings};

fn main() {
    let mut output = vec![];

    {
        let mut rewriter = HtmlRewriter::try_new(
            Settings {
                element_content_handlers: vec![
                    element!("a[href]", |el| {
                        let href = el
                            .get_attribute("href")
                            .unwrap()
                            .replace("http:", "https:");

                        el.set_attribute("href", &href).unwrap();

                        Ok(())
                    })
                ],
                ..Settings::default()
            },
            |c: &[u8]| output.extend_from_slice(c)
        ).unwrap();

        rewriter.write(b"<div><a href=").unwrap();
        rewriter.write(b"http://example.com>").unwrap();
        rewriter.write(b"</a></div>").unwrap();
        rewriter.end().unwrap();
    }

    assert_eq!(
        String::from_utf8(output).unwrap(),
        r#"<div><a href="https://example.com"></a></div>"#
    );
}
```

# License

BSD licensed. See the [LICENSE](LICENSE) file for details.
