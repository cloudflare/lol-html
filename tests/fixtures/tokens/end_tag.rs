use cool_thing::{Bytes, EndTag, ContentType};

test_fixture!("End tag token", {
    test("Serialization", {
        serialization_test!(
            "</div foo=bar>",
            EndTag,
            &[
                ("Parsed", Box::new(|_, _| {}), "</div foo=bar>"),
                (
                    "Modified name",
                    Box::new(|t, encoding| {
                        t.set_name(Bytes::from_str("span", encoding));
                    }),
                    "</span>",
                ),
                (
                    "With prepends and appends",
                    Box::new(|t, _| {
                        t.before("<span>", ContentType::Text);
                        t.before("<div>Hey</div>", ContentType::Html);
                        t.before("<foo>", ContentType::Html);
                        t.after("</foo>", ContentType::Html);
                        t.after("<!-- 42 -->", ContentType::Html);
                        t.after("<foo & bar>", ContentType::Text);
                    }),
                    concat!(
                        "&lt;span&gt;<div>Hey</div><foo></div foo=bar>",
                        "&lt;foo &amp; bar&gt;<!-- 42 --></foo>",
                    )
                ),
                (
                    "Removed",
                    Box::new(|t, _| {
                        assert!(!t.removed());

                        t.remove();

                        assert!(t.removed());

                        t.before("<before>", ContentType::Html);
                        t.after("<after>", ContentType::Html);
                    }),
                    "<before><after>",
                ),
                (
                    "Replaced",
                    Box::new(|t, _| {
                        t.before("<before>", ContentType::Html);
                        t.after("<after>", ContentType::Html);

                        assert!(!t.removed());

                        t.replace("<div></div>", ContentType::Html);
                        t.replace("<!--42-->", ContentType::Html);
                        t.replace("<foo & bar>", ContentType::Text);

                        assert!(t.removed());
                    }),
                    "<before><div></div><!--42-->&lt;foo &amp; bar&gt;<after>",
                ),
            ]
        );
    });
});
