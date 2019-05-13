use cool_thing::{Bytes, ContentType, EndTag, Mutations};

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
                    Box::new(|t, encoding| {
                        let mut m = Mutations::new(encoding);

                        m.before("<span>", ContentType::Text);
                        m.before("<div>Hey</div>", ContentType::Html);
                        m.before("<foo>", ContentType::Html);
                        m.after("</foo>", ContentType::Html);
                        m.after("<!-- 42 -->", ContentType::Html);
                        m.after("<foo & bar>", ContentType::Text);

                        t.mutations = m;
                    }),
                    concat!(
                        "&lt;span&gt;<div>Hey</div><foo></div foo=bar>",
                        "&lt;foo &amp; bar&gt;<!-- 42 --></foo>",
                    )
                ),
                (
                    "Removed",
                    Box::new(|t, encoding| {
                        let mut m = Mutations::new(encoding);

                        m.remove();
                        m.before("<before>", ContentType::Html);
                        m.after("<after>", ContentType::Html);

                        t.mutations = m;
                    }),
                    "<before><after>",
                ),
                (
                    "Replaced with text",
                    Box::new(|t, _| {
                        t.mutations.before("<before>", ContentType::Html);
                        t.mutations.after("<after>", ContentType::Html);

                        assert!(!t.mutations.removed());

                        t.mutations.replace("<div></div>", ContentType::Html);
                        t.mutations.replace("<!--42-->", ContentType::Html);
                        t.mutations.replace("<foo & bar>", ContentType::Text);

                        assert!(t.mutations.removed());
                    }),
                    "<before>&lt;foo &amp; bar&gt;<after>",
                ),
                (
                    "Replaced with HTML",
                    Box::new(|t, _| {
                        t.mutations.before("<before>", ContentType::Html);
                        t.mutations.after("<after>", ContentType::Html);

                        assert!(!t.mutations.removed());

                        t.mutations.replace("<div></div>", ContentType::Html);
                        t.mutations.replace("<!--42-->", ContentType::Html);
                        t.mutations.replace("<foo & bar>", ContentType::Html);

                        assert!(t.mutations.removed());
                    }),
                    "<before><foo & bar><after>",
                ),
            ]
        );
    });
});
