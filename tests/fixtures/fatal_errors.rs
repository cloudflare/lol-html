use cool_thing::*;
use std::convert::TryFrom;
use crate::harness::Input;
use encoding_rs::UTF_8;
use failure::format_err;

fn create_rewriter<O: OutputSink>(
    buffer_capacity: usize,
    output_sink: O
) -> HtmlRewriter<'static, O> {
    HtmlRewriter::try_from(Settings {
        element_content_handlers: vec![(
            &"*".parse().unwrap(),
            ElementContentHandlers::default().element(|_| {Ok(())}),
        )],
        document_content_handlers: vec![],
        encoding: "utf-8",
        buffer_capacity,
        output_sink,
    })
    .unwrap()
}

test_fixture!("Fatal errors", {
    test("Buffer capacity limit", {
        const BUFFER_SIZE: usize = 20;

        let mut rewriter = create_rewriter(BUFFER_SIZE, |_: &[u8]| {});

        // Use two chunks for the stream to force the usage of the buffer and
        // make sure to overflow it.
        let chunk_1 = format!("<img alt=\"{}", "l".repeat(BUFFER_SIZE / 2));
        let chunk_2 = format!("{}\" />", "r".repeat(BUFFER_SIZE / 2));

        rewriter.write(chunk_1.as_bytes()).unwrap();

        let write_err = rewriter.write(chunk_2.as_bytes()).unwrap_err();

        let buffer_capacity_err = write_err
            .find_root_cause()
            .downcast_ref::<BufferCapacityExceededError>()
            .unwrap();

        assert_eq!(
            *buffer_capacity_err,
            BufferCapacityExceededError { capacity: 20 }
        );
    });

    test("Write after end",
        expect_panic: "Data was written into the stream after it has ended.",
    {
        let mut rewriter = create_rewriter(512, |_: &[u8]| {});

        rewriter.end().unwrap();
        rewriter.write(b"foo").unwrap();
    });

    test("End twice",
        expect_panic: "Stream was ended twice.",
    {
        let mut rewriter = create_rewriter(512, |_: &[u8]| {});

        rewriter.end().unwrap();
        rewriter.end().unwrap();
    });

    test("Poisoning after unrecovarable error",
        expect_panic: "Attempt to use the HtmlRewriter after a fatal error.",
    {
        const BUFFER_SIZE: usize = 10;

        let mut rewriter = create_rewriter(BUFFER_SIZE, |_: &[u8]| {});
        let chunk = format!("<img alt=\"{}", "l".repeat(BUFFER_SIZE));

        rewriter.write(chunk.as_bytes()).unwrap_err();
        rewriter.end().unwrap_err();
    });

    test("Content handler errors propagation", {
        macro_rules! assert_err {
            ($element_handlers:expr, $document_handlers:expr, $expected_err:expr) => {{
                let mut rewriter = HtmlRewriter::try_from(Settings {
                    element_content_handlers: vec![(
                        &"*".parse().unwrap(),
                        $element_handlers
                    )],
                    document_content_handlers: vec![$document_handlers],
                    encoding: "utf-8",
                    buffer_capacity: 2048,
                    output_sink: |_: &[u8]| {},
                })
                .unwrap();

                let mut input = Input::from(
                    String::from("<!--doc comment--> Doc text <div><!--el comment-->El text</div>")
                );

                input.init(UTF_8, false).unwrap();

                let mut err = None;

                for chunk in input.chunks() {
                    match rewriter.write(chunk) {
                        Ok(_) => (),
                        Err(e) => {
                            err = Some(e);
                            break;
                        }
                    }
                }

                if err.is_none(){
                    match rewriter.end() {
                        Ok(_) => (),
                        Err(e) => err = Some(e)
                    }
                }

                let err = format!("{}", err.expect("Error expected"));

                assert_eq!(err, $expected_err);
            }};
        }

        assert_err!(
            ElementContentHandlers::default(),
            DocumentContentHandlers::default().comments(|_| {
                Err(format_err!("Error in doc comment handler"))
            }),
            "Error in doc comment handler"
        );

        assert_err!(
            ElementContentHandlers::default(),
            DocumentContentHandlers::default().text(|_| {
                Err(format_err!("Error in doc text handler"))
            }),
            "Error in doc text handler"
        );

        assert_err!(
            ElementContentHandlers::default(),
            DocumentContentHandlers::default().text(|_| {
                Err(format_err!("Error in doctype handler"))
            }),
            "Error in doctype handler"
        );

        assert_err!(
            ElementContentHandlers::default().element(|_| {
                Err(format_err!("Error in element handler"))
            }),
            DocumentContentHandlers::default(),
            "Error in element handler"
        );

        assert_err!(
            ElementContentHandlers::default().comments(|_| {
                Err(format_err!("Error in element comment handler"))
            }),
            DocumentContentHandlers::default(),
            "Error in element comment handler"
        );

        assert_err!(
            ElementContentHandlers::default().text(|_| {
                Err(format_err!("Error in element text handler"))
            }),
            DocumentContentHandlers::default(),
            "Error in element text handler"
        );
    });
});
