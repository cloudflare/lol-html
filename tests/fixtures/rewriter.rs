use cool_thing::{
    HtmlRewriterBuilder, EncodingError, DocumentContentHandlers, ElementContentHandlers,
    HtmlRewriter, Bytes, OutputSink, ContentType
};
use encoding_rs::Encoding;
use crate::harness::{ASCII_COMPATIBLE_ENCODINGS, TestOutput};

fn write_chunks<O: OutputSink>(rewriter: &mut HtmlRewriter<'_, O>, encoding: &'static Encoding, chunks: &[&str]) {
    for chunk in chunks {
        rewriter.write(&Bytes::from_str(chunk, encoding)).unwrap();
    }

    rewriter.end().unwrap();
}

test_fixture!("Rewriter", {
    test("Unknown encoding", {
        let err = HtmlRewriterBuilder::default()
            .build("hey-yo", |_: &[u8]|{})
            .unwrap_err();

        assert_eq!(err, EncodingError::UnknownEncoding);
    });

    test("Non-ASCII compatible encoding", {
        let err = HtmlRewriterBuilder::default()
            .build("utf-16be", |_: &[u8]|{})
            .unwrap_err();

        assert_eq!(err, EncodingError::NonAsciiCompatibleEncoding);
    });

    test("Doctype info", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let mut doctypes = Vec::default();

            {
                let mut rewriter = HtmlRewriterBuilder::default()
                    .on_document(
                        DocumentContentHandlers::default()
                            .doctype(|d| doctypes.push((d.name(), d.public_id(), d.system_id()))),
                    )
                    .build(enc.name(), |_: &[u8]| {})
                    .unwrap();

                write_chunks(&mut rewriter, enc, &[
                    "<!doctype html1>",
                    "<!-- test --><div>",
                    r#"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN" "#,
                    r#""http://www.w3.org/TR/html4/strict.dtd">"#,
                    "</div><!DoCtYPe "
                ]);
            }

            assert_eq!(doctypes, &[
                (Some("html1".into()), None, None),
                (
                    Some("html".into()),
                    Some("-//W3C//DTD HTML 4.01//EN".into()),
                    Some("http://www.w3.org/TR/html4/strict.dtd".into())
                ),
                (None, None, None),
            ]);
        }
    });

    test("Rewrite all element start tags", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let actual: String = {
                let mut output = TestOutput::new(enc);

                let mut rewriter = HtmlRewriterBuilder::default()
                    .on("*", ElementContentHandlers::default()
                        .element(|el| {
                            el.set_attribute("foo", "bar").unwrap();
                            el.prepend("<test></test>", ContentType::Html);
                        })
                    )
                    .unwrap()
                    .build(enc.name(), |c: &[u8]| output.push(c))
                    .unwrap();

                write_chunks(&mut rewriter, enc, &[
                    "<!doctype html>\n",
                    "<html>\n",
                    "   <head></head>\n",
                    "   <body>\n",
                    "       <div>Test</div>\n",
                    "   </body>\n",
                    "</html>",
                ]);

                output.into()
            };

            assert_eq!(actual, concat!(
                "<!doctype html>\n",
                "<html foo=\"bar\"><test></test>\n",
                "   <head foo=\"bar\"><test></test></head>\n",
                "   <body foo=\"bar\"><test></test>\n",
                "       <div foo=\"bar\"><test></test>Test</div>\n",
                "   </body>\n",
                "</html>",
            ));
        }
    });

    test("Rewrite document content", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let actual: String = {
                let mut output = TestOutput::new(enc);

                let mut rewriter = HtmlRewriterBuilder::default()
                    .on_document(DocumentContentHandlers::default()
                        .comments(|c| {
                            c.set_text(&(c.text() + "1337")).unwrap();
                        })
                        .text(|c| {
                            if c.last_in_text_node() {
                                c.insert_after("BAZ", ContentType::Text);
                            }
                        })
                    )
                    .build(enc.name(), |c: &[u8]| output.push(c))
                    .unwrap();

                write_chunks(&mut rewriter, enc, &[
                    "<!doctype html>\n",
                    "<!-- hey -->\n",
                    "<html>\n",
                    "   <head><!-- aloha --></head>\n",
                    "   <body>\n",
                    "       <div>Test</div>\n",
                    "   </body>\n",
                    "   <!-- bonjour -->\n",
                    "</html>Pshhh",
                ]);

                output.into()
            };

            assert_eq!(actual, concat!(
               "<!doctype html>\nBAZ",
                "<!-- hey 1337-->\nBAZ",
                "<html>\n",
                "   BAZ<head><!-- aloha 1337--></head>\n",
                "   BAZ<body>\n",
                "       BAZ<div>TestBAZ</div>\n",
                "   BAZ</body>\n",
                "   BAZ<!-- bonjour 1337-->\nBAZ",
                "</html>PshhhBAZ",
            ));
        }
    });
});
