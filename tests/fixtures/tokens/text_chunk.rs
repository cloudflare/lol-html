use cool_thing::{TextChunk, ContentType};

test_fixture!("Text chunk token", {
    test("Serialization", {
        let src =
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
             incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud \
             exercitation & ullamco laboris nisi ut aliquip ex ea commodo > consequat.";

        serialization_test!(
            src,
            TextChunk,
            &[
                ("Parsed", Box::new(|_, _| {}), src),
                (
                    "With prepends and appends",
                    Box::new(|c, _| {
                        c.insert_before("<span>", ContentType::Text);
                        c.insert_before("<div>Hey</div>", ContentType::Html);
                        c.insert_before("<foo>", ContentType::Html);
                        c.insert_after("</foo>", ContentType::Html);
                        c.insert_after("<!-- 42 -->", ContentType::Html);
                        c.insert_after("<foo & bar>", ContentType::Text);
                    }),
                    concat!(
                        "&lt;span&gt;<div>Hey</div><foo>",
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod \
                         tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim \
                         veniam, quis nostrud exercitation & ullamco laboris nisi ut aliquip \
                         ex ea commodo > consequat.",
                        "&lt;foo &amp; bar&gt;<!-- 42 --></foo>"
                    )
                ),
                (
                    "Removed",
                    Box::new(|c, _| {
                        assert!(!c.removed());

                        c.remove();

                        assert!(c.removed());

                        c.insert_before("<before>", ContentType::Html);
                        c.insert_after("<after>", ContentType::Html);
                    }),
                    "<before><after>",
                ),
                (
                    "Replaced",
                    Box::new(|c, _| {
                        c.insert_before("<before>", ContentType::Html);
                        c.insert_after("<after>", ContentType::Html);

                        assert!(!c.removed());

                        c.replace("<div></div>", ContentType::Html);
                        c.replace("<!--42-->", ContentType::Html);

                        assert!(c.removed());
                    }),
                    "<before><div></div><!--42--><after>",
                ),
            ]
        );
    });
});
