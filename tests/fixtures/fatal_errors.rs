use cool_thing::*;
use std::convert::TryFrom;

fn create_rewriter<O: OutputSink>(
    buffer_capacity: usize,
    output_sink: O
) -> HtmlRewriter<'static, O> {
    HtmlRewriter::try_from(Settings {
        element_content_handlers: vec![(
            &"*".parse().unwrap(),
            ElementContentHandlers::default().element(|_| {}),
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
});
