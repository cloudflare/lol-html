use cool_thing::{Bytes, StartTag, ContentType};

test_fixture!("Start tag token", {
    test("Serialization", {
        serialization_test!(
            r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            StartTag,
            &[
            (
                "Parsed",
                Box::new(|_, _| {}),
                r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            ),
            (
                "Modified name",
                Box::new(|t, encoding| {
                    t.set_name(Bytes::from_str("div", encoding));
                }),
                r#"<div a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            ),
            (
                "Modified single quotted attribute value",
                Box::new(|t, _| {
                    t.set_attribute("a1", r#"foo " bar " baz42"#).unwrap();
                }),
                r#"<a a1="foo &quot; bar &quot; baz42" a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            ),
            (
                "Modified double quotted attribute value",
                Box::new(|t, _| {
                    t.set_attribute("a2", "foo ' bar ' baz42").unwrap();
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz42" a3=foo/bar a4>"#,
            ),
            (
                "Modified unquotted attribute value",
                Box::new(|t, _| {
                    t.set_attribute("a3", "foo/bar42").unwrap();
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3="foo/bar42" a4>"#,
            ),
            (
                "Set value for an attribute without a value",
                Box::new(|t, _| {
                    t.set_attribute("a4", "42").unwrap();
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4="42">"#,
            ),
            (
                "Add attribute",
                Box::new(|t, _| {
                    t.set_attribute("a5", r#"42'"42"#).unwrap();
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4 a5="42'&quot;42">"#,
            ),
            (
                "With self-closing flag",
                Box::new(|t, _| {
                    t.set_self_closing(true);
                }),
                r#"<a a1='foo " bar " baz' a2="foo ' bar ' baz" a3=foo/bar a4 />"#,
            ),
            (
                "Remove non-existent attribute",
                Box::new(|t, _| t.remove_attribute("a5")),
                r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
            ),
            (
                "Without attributes",
                Box::new(|t, _| {
                    for name in &["a1", "a2", "a3", "a4"] {
                        t.remove_attribute(name);
                    }
                }),
                r#"<a>"#,
            ),
            (
                "Without attributes self-closing",
                Box::new(|t, _| {
                    t.set_self_closing(true);

                    for name in &["a1", "a2", "a3", "a4"] {
                        t.remove_attribute(name);
                    }
                }),
                r#"<a/>"#,
            ),
            (
                "With prepends and appends",
                Box::new(|t, _| {
                    t.insert_before("<span>", ContentType::Text);
                    t.insert_before("<div>Hey</div>", ContentType::Html);
                    t.insert_before("<foo>", ContentType::Html);
                    t.insert_after("</foo>", ContentType::Html);
                    t.insert_after("<!-- 42 -->", ContentType::Html);
                    t.insert_after("<foo & bar>", ContentType::Text);
                }),
                concat!(
                    "&lt;span&gt;<div>Hey</div><foo>",
                    r#"<a a1='foo " bar " baz' / a2="foo ' bar ' baz" a3=foo/bar a4>"#,
                    "&lt;foo &amp; bar&gt;<!-- 42 --></foo>"
                ),
            ),
            (
                "Removed",
                Box::new(|t, _| {
                    assert!(!t.removed());

                    t.remove();

                    assert!(t.removed());

                    t.insert_before("<before>", ContentType::Html);
                    t.insert_after("<after>", ContentType::Html);
                }),
                "<before><after>",
            ),
            (
                "Replaced",
                Box::new(|t, _| {
                    t.insert_before("<before>", ContentType::Html);
                    t.insert_after("<after>", ContentType::Html);

                    assert!(!t.removed());

                    t.replace("<div></div>", ContentType::Html);
                    t.replace("<!--42-->", ContentType::Html);
                    t.replace("<foo & bar>", ContentType::Text);

                    assert!(t.removed());
                }),
                "<before><div></div><!--42-->&lt;foo &amp; bar&gt;<after>",
            ),
        ]);
    });
});
