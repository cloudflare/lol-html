
use crate::harness::{Output, ASCII_COMPATIBLE_ENCODINGS};
use cool_thing::{
    Bytes, ContentType, DocumentContentHandlers, ElementContentHandlers, EncodingError,
    HtmlRewriter, OutputSink, Settings
};
use encoding_rs::Encoding;
use std::convert::TryFrom;

fn write_chunks<O: OutputSink>(
    rewriter: &mut HtmlRewriter<O>,
    encoding: &'static Encoding,
    chunks: &[&str],
) {
    for chunk in chunks {
        rewriter.write(&Bytes::from_str(chunk, encoding)).unwrap();
    }

    rewriter.end().unwrap();
}

test_fixture!("Rewriter", {
    test("Unknown encoding", {
        let err = HtmlRewriter::try_from(Settings {
            element_content_handlers: vec![],
            document_content_handlers: vec![],
            encoding: "hey-yo",
            buffer_capacity: 42,
            output_sink: |_: &[u8]| {}
        }).unwrap_err();

        assert_eq!(err, EncodingError::UnknownEncoding);
    });

    test("Non-ASCII compatible encoding", {
        let err = HtmlRewriter::try_from(Settings {
            element_content_handlers: vec![],
            document_content_handlers: vec![],
            encoding: "utf-16be",
            buffer_capacity: 42,
            output_sink: |_: &[u8]| {}
        }).unwrap_err();

        assert_eq!(err, EncodingError::NonAsciiCompatibleEncoding);
    });

    test("Doctype info", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let mut doctypes = Vec::default();

            {
                let mut rewriter = HtmlRewriter::try_from(Settings{
                    element_content_handlers: vec![],
                    document_content_handlers: vec![DocumentContentHandlers::default()
                        .doctype(|d| doctypes.push((d.name(), d.public_id(), d.system_id())))],
                    encoding: enc.name(),
                    buffer_capacity: 2048,
                    output_sink: |_: &[u8]| {},
                })
                .unwrap();

                write_chunks(
                    &mut rewriter,
                    enc,
                    &[
                        "<!doctype html1>",
                        "<!-- test --><div>",
                        r#"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN" "#,
                        r#""http://www.w3.org/TR/html4/strict.dtd">"#,
                        "</div><!DoCtYPe ",
                    ],
                );
            }

            assert_eq!(
                doctypes,
                &[
                    (Some("html1".into()), None, None),
                    (
                        Some("html".into()),
                        Some("-//W3C//DTD HTML 4.01//EN".into()),
                        Some("http://www.w3.org/TR/html4/strict.dtd".into())
                    ),
                    (None, None, None),
                ]
            );
        }
    });

    test("Rewrite all element start tags", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let actual: String = {
                let mut output = Output::new(enc);

                let mut rewriter = HtmlRewriter::try_from(Settings{
                    element_content_handlers: vec![(
                        &"*".parse().unwrap(),
                        ElementContentHandlers::default().element(|el| {
                            el.set_attribute("foo", "bar").unwrap();
                            el.prepend("<test></test>", ContentType::Html);
                        }),
                    )],
                    document_content_handlers: vec![],
                    encoding: enc.name(),
                    buffer_capacity: 2048,
                    output_sink: |c: &[u8]| output.push(c),
                })
                .unwrap();

                write_chunks(
                    &mut rewriter,
                    enc,
                    &[
                        "<!doctype html>\n",
                        "<html>\n",
                        "   <head></head>\n",
                        "   <body>\n",
                        "       <div>Test</div>\n",
                        "   </body>\n",
                        "</html>",
                    ],
                );

                output.into()
            };

            assert_eq!(
                actual,
                concat!(
                    "<!doctype html>\n",
                    "<html foo=\"bar\"><test></test>\n",
                    "   <head foo=\"bar\"><test></test></head>\n",
                    "   <body foo=\"bar\"><test></test>\n",
                    "       <div foo=\"bar\"><test></test>Test</div>\n",
                    "   </body>\n",
                    "</html>",
                )
            );
        }
    });

    test("Rewrite document content", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let actual: String = {
                let mut output = Output::new(enc);

                let mut rewriter = HtmlRewriter::try_from(Settings{
                    element_content_handlers: vec![],
                    document_content_handlers:vec![DocumentContentHandlers::default()
                        .comments(|c| {
                            c.set_text(&(c.text() + "1337")).unwrap();
                        })
                        .text(|c| {
                            if c.last_in_text_node() {
                                c.after("BAZ", ContentType::Text);
                            }
                        })],
                    encoding: enc.name(),
                    buffer_capacity: 2048,
                    output_sink: |c: &[u8]| output.push(c),
                })
                .unwrap();

                write_chunks(
                    &mut rewriter,
                    enc,
                    &[
                        "<!doctype html>\n",
                        "<!-- hey -->\n",
                        "<html>\n",
                        "   <head><!-- aloha --></head>\n",
                        "   <body>\n",
                        "       <div>Test</div>\n",
                        "   </body>\n",
                        "   <!-- bonjour -->\n",
                        "</html>Pshhh",
                    ],
                );

                output.into()
            };

            assert_eq!(
                actual,
                concat!(
                    "<!doctype html>\nBAZ",
                    "<!-- hey 1337-->\nBAZ",
                    "<html>\n",
                    "   BAZ<head><!-- aloha 1337--></head>\n",
                    "   BAZ<body>\n",
                    "       BAZ<div>TestBAZ</div>\n",
                    "   BAZ</body>\n",
                    "   BAZ<!-- bonjour 1337-->\nBAZ",
                    "</html>PshhhBAZ",
                )
            );
        }
    });
});
