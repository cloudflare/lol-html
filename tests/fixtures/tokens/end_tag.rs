use cool_thing::base::Bytes;
use cool_thing::content::EndTag;

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
                        t.before("<div>Hey</div>");
                        t.before("<foo>");
                        t.after("</foo>");
                        t.after("<!-- 42 -->");
                    }),
                    "<div>Hey</div><foo></div foo=bar><!-- 42 --></foo>",
                ),
                (
                    "Removed",
                    Box::new(|t, _| {
                        assert!(!t.removed());

                        t.remove();

                        assert!(t.removed());

                        t.before("<before>");
                        t.after("<after>");
                    }),
                    "<before><after>",
                ),
                (
                    "Replaced",
                    Box::new(|t, _| {
                        t.before("<before>");
                        t.after("<after>");

                        assert!(!t.removed());

                        t.replace("<div></div>");
                        t.replace("<!--42-->");

                        assert!(t.removed());
                    }),
                    "<before><div></div><!--42--><after>",
                ),
            ]
        );
    });
});
