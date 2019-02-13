use cool_thing::token::TextChunk;

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
                ("Parsed", Box::new(|_| {}), src),
                (
                    "With prepends and appends",
                    Box::new(|c| {
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
                    Box::new(|c| {
                        c.before("<before>");
                        c.after("<after>");
                        c.remove();
                    }),
                    "<before><after>",
                ),
                (
                    "Replaced",
                    Box::new(|c| {
                        c.before("<before>");
                        c.after("<after>");
                        c.replace("<div></div>");
                        c.replace("<!--42-->");
                    }),
                    "<before><div></div><!--42--><after>",
                ),
            ]
        );
    });
});
