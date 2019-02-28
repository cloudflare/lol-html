use cool_thing::content::TextChunk;

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
                        c.before("<div>Hey</div>");
                        c.before("<foo>");
                        c.after("</foo>");
                        c.after("<!-- 42 -->");
                    }),
                    concat!(
                        "<div>Hey</div><foo>",
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod \
                         tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim \
                         veniam, quis nostrud exercitation & ullamco laboris nisi ut aliquip \
                         ex ea commodo > consequat.",
                        "<!-- 42 --></foo>"
                    )
                ),
                (
                    "Removed",
                    Box::new(|c, _| {
                        assert!(!c.removed());

                        c.remove();

                        assert!(c.removed());

                        c.before("<before>");
                        c.after("<after>");
                    }),
                    "<before><after>",
                ),
                (
                    "Replaced",
                    Box::new(|c, _| {
                        c.before("<before>");
                        c.after("<after>");

                        assert!(!c.removed());

                        c.replace("<div></div>");
                        c.replace("<!--42-->");

                        assert!(c.removed());
                    }),
                    "<before><div></div><!--42--><after>",
                ),
            ]
        );
    });
});
