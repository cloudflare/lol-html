use lol_html::*;

define_group!(
    "Parsing",
    [
        ("Tag scanner", Settings::new()),
        (
            "Lexer",
            // NOTE: this switches parser to the lexer mode and doesn't
            // trigger token production for anything, except doctype. So,
            // we can get relatively fair comparison.
            Settings::new().append_document_content_handler(doctype!(noop_handler!()))
        ),
        (
            "Text rewritable unit parsing and decoding",
            // NOTE: this is the biggest bottleneck part of the parser and rewriter.
            // It's not guaranteed that chunks that come over the wire contain decodable
            // sequence of bytes for the given character encoding. So, if there is a text
            // handler in the selector matching scope, we need to slice and decode all
            // incoming chunks to produce correct text chunk rewritable units.
            Settings::new().append_document_content_handler(doc_text!(noop_handler!()))
        )
    ]
);

pub fn many_attributes(c: &mut Criterion) {
    let input = format!("<div{}>body</div>", " a".repeat(65536));
    let mut group = c.benchmark_group("Many attributes");
    group.throughput(Throughput::Bytes(input.len() as u64));
    for chunk_size in [512, 4096, 65536] {
        group.bench_with_input(
            BenchmarkId::from_parameter(chunk_size),
            &chunk_size,
            |b, &chunk_size| {
                b.iter(|| {
                    let settings =
                        Settings::new().append_element_content_handler(element!("div", |_| Ok(())));
                    let mut rewriter = HtmlRewriter::new(settings, |bytes: &[u8]| {
                        std::hint::black_box(bytes);
                    });
                    for chunk in input.as_bytes().chunks(chunk_size) {
                        rewriter.write(chunk).unwrap();
                    }
                    rewriter.end().unwrap();
                });
            },
        );
    }
    group.finish();
}
