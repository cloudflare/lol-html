use cool_thing::{ContentType, TextChunk};

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
                        c.before("<span>", ContentType::Text);
                        c.before("<div>Hey</div>", ContentType::Html);
                        c.before("<foo>", ContentType::Html);
                        c.after("</foo>", ContentType::Html);
                        c.after("<!-- 42 -->", ContentType::Html);
                        c.after("<foo & bar>", ContentType::Text);
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

                        c.before("<before>", ContentType::Html);
                        c.after("<after>", ContentType::Html);
                    }),
                    "<before><after>",
                ),
                (
                    "Replaced with text",
                    Box::new(|c, _| {
                        c.before("<before>", ContentType::Html);
                        c.after("<after>", ContentType::Html);

                        assert!(!c.removed());

                        c.replace("<div></div>", ContentType::Html);
                        c.replace("<!--42-->", ContentType::Html);
                        c.replace("<foo & bar>", ContentType::Text);

                        assert!(c.removed());
                    }),
                    "<before>&lt;foo &amp; bar&gt;<after>",
                ),
                (
                    "Replaced with HTML",
                    Box::new(|c, _| {
                        c.before("<before>", ContentType::Html);
                        c.after("<after>", ContentType::Html);

                        assert!(!c.removed());

                        c.replace("<div></div>", ContentType::Html);
                        c.replace("<!--42-->", ContentType::Html);
                        c.replace("<foo & bar>", ContentType::Html);

                        assert!(c.removed());
                    }),
                    "<before><foo & bar><after>",
                ),
            ]
        );
    });
});
