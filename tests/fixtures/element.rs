
use crate::harness::{Output, ASCII_COMPATIBLE_ENCODINGS};
use cool_thing::{
    AttributeNameError, ContentType, Element, ElementContentHandlers, HtmlRewriter, TagNameError,
};
use encoding_rs::{Encoding, EUC_JP, UTF_8};

fn rewrite_element(
    html: &str,
    encoding: &'static Encoding,
    selector: &str,
    mut handler: impl FnMut(&mut Element),
) -> String {
    let mut handler_called = false;
    let mut output = Output::new(encoding);

    {
        let mut rewriter = HtmlRewriter::try_new(
            vec![
                (
                    &selector.parse().unwrap(),
                    ElementContentHandlers::default().element(|el| {
                        handler_called = true;
                        handler(el);
                    }),
                ),
                // NOTE: used to test inner content removal
                (
                    &"inner-remove-me".parse().unwrap(),
                    ElementContentHandlers::default().element(|el| {
                        el.before("[before: should be removed]", ContentType::Text);
                        el.after("[after: should be removed]", ContentType::Text);
                        el.append("[append: should be removed]", ContentType::Text);
                        el.before("[before: should be removed]", ContentType::Text);
                    }),
                ),
            ],
            vec![],
            encoding.name(),
            |c: &[u8]| output.push(c),
        )
        .unwrap();

        rewriter.write(html.as_bytes()).unwrap();
        rewriter.end().unwrap();
    }

    assert!(handler_called);

    output.into()
}

test_fixture!("Element rewritable unit", {
    test("Empty tag name", {
        rewrite_element("<div>", UTF_8, "div", |el| {
            let err = el.set_tag_name("").unwrap_err();

            assert_eq!(err, TagNameError::Empty);
        });
    });

    test("Forbidden characters in tag name", {
        rewrite_element("<div>", UTF_8, "div", |el| {
            for &ch in &[' ', '\n', '\r', '\t', '\x0C', '/', '>'] {
                let err = el.set_tag_name(&format!("foo{}bar", ch)).unwrap_err();

                assert_eq!(err, TagNameError::ForbiddenCharacter(ch));
            }
        });
    });

    test("Encoding-unmappable characters in tag name", {
        rewrite_element("<div>", EUC_JP, "div", |el| {
            let err = el.set_tag_name("foo\u{00F8}bar").unwrap_err();

            assert_eq!(err, TagNameError::UnencodableCharacter);
        });
    });

    test("Invalid first character of tag name", {
        rewrite_element("<div>", UTF_8, "div", |el| {
            let err = el.set_tag_name("1foo").unwrap_err();

            assert_eq!(err, TagNameError::InvalidFirstCharacter);
        });
    });

    test("Empty attribute name", {
        rewrite_element("<div>", UTF_8, "div", |el| {
            let err = el.set_attribute("", "").unwrap_err();

            assert_eq!(err, AttributeNameError::Empty);
        });
    });

    test("Forbidden characters in attribute name", {
        rewrite_element("<div>", UTF_8, "div", |el| {
            for &ch in &[' ', '\n', '\r', '\t', '\x0C', '/', '>', '='] {
                let err = el.set_attribute(&format!("foo{}bar", ch), "").unwrap_err();

                assert_eq!(err, AttributeNameError::ForbiddenCharacter(ch));
            }
        });
    });

    test("Encoding-unmappable characters in attribute name", {
        rewrite_element("<div>", EUC_JP, "div", |el| {
            let err = el.set_attribute("foo\u{00F8}bar", "").unwrap_err();

            assert_eq!(err, AttributeNameError::UnencodableCharacter);
        });
    });

    test("Tag name getter and setter", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<Foo><div><span></span></div></foo>", enc, "foo", |el| {
                assert_eq!(el.tag_name(), "foo", "Encoding: {}", enc.name());

                el.set_tag_name("BaZ").unwrap();

                assert_eq!(el.tag_name(), "baz", "Encoding: {}", enc.name());
            });

            assert_eq!(output, "<BaZ><div><span></span></div></BaZ>");
        }
    });

    test("Attribute list", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, "foo", |el| {
                assert_eq!(el.attributes().len(), 2, "Encoding: {}", enc.name());
                assert_eq!(
                    el.attributes()[0].name(),
                    "foo1",
                    "Encoding: {}",
                    enc.name()
                );
                assert_eq!(
                    el.attributes()[1].name(),
                    "foo2",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.attributes()[0].value(),
                    "Bar1",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.attributes()[1].value(),
                    "Bar2",
                    "Encoding: {}",
                    enc.name()
                );
            });
        }
    });

    test("Get attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, "foo", |el| {
                assert_eq!(
                    el.get_attribute("fOo1").unwrap(),
                    "Bar1",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.get_attribute("Foo1").unwrap(),
                    "Bar1",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.get_attribute("FOO2").unwrap(),
                    "Bar2",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(
                    el.get_attribute("foo2").unwrap(),
                    "Bar2",
                    "Encoding: {}",
                    enc.name()
                );

                assert_eq!(el.get_attribute("foo3"), None, "Encoding: {}", enc.name());
            });
        }
    });

    test("Has attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, "foo", |el| {
                assert!(el.has_attribute("FOo1"), "Encoding: {}", enc.name());
                assert!(el.has_attribute("foo1"), "Encoding: {}", enc.name());
                assert!(el.has_attribute("FOO2"), "Encoding: {}", enc.name());
                assert!(!el.has_attribute("foo3"), "Encoding: {}", enc.name());
            });
        }
    });

    test("Set attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<div>", enc, "div", |el| {
                el.set_attribute("Foo", "Bar1").unwrap();

                assert_eq!(
                    el.get_attribute("foo").unwrap(),
                    "Bar1",
                    "Encoding: {}",
                    enc.name()
                );

                el.set_attribute("fOO", "Bar2").unwrap();

                assert_eq!(
                    el.get_attribute("foo").unwrap(),
                    "Bar2",
                    "Encoding: {}",
                    enc.name()
                );
            });
        }
    });

    test("Remove attribute", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            rewrite_element("<Foo Foo1=Bar1 Foo2=Bar2>", enc, "foo", |el| {
                el.remove_attribute("Unknown");

                assert_eq!(el.attributes().len(), 2, "Encoding: {}", enc.name());

                el.remove_attribute("Foo1");

                assert_eq!(el.attributes().len(), 1, "Encoding: {}", enc.name());
                assert_eq!(el.get_attribute("foo1"), None, "Encoding: {}", enc.name());

                el.remove_attribute("FoO2");

                assert!(el.attributes().is_empty(), "Encoding: {}", enc.name());
                assert_eq!(el.get_attribute("foo2"), None, "Encoding: {}", enc.name());
            });
        }
    });

    test("Insert content before", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.before("<img>", ContentType::Html);
                el.before("<img>", ContentType::Text);
            });

            assert_eq!(output, "<div><img>&lt;img&gt;<span>Hi</span></div>");
        }
    });

    test("Prepend content", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.prepend("<img>", ContentType::Html);
                el.prepend("<img>", ContentType::Text);
            });

            assert_eq!(output, "<div><span>&lt;img&gt;<img>Hi</span></div>");
        }
    });

    test("Append content", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.append("<img>", ContentType::Html);
                el.append("<img>", ContentType::Text);
            });

            assert_eq!(output, "<div><span>Hi<img>&lt;img&gt;</span></div>");
        }
    });

    test("Insert content after", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.after("<img>", ContentType::Html);
                el.after("<img>", ContentType::Text);
            });

            assert_eq!(output, "<div><span>Hi</span>&lt;img&gt;<img></div>");
        }
    });

    test("Set inner content", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.set_inner_content("<img>", ContentType::Html);
                    el.set_inner_content("<img>", ContentType::Text);
                },
            );

            assert_eq!(output, "<div><span>&lt;img&gt;</span></div>");

            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.set_inner_content("<img>", ContentType::Text);
                    el.set_inner_content("<img>", ContentType::Html);
                },
            );

            assert_eq!(output, "<div><span><img></span></div>");
        }
    });

    test("Replace", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.replace("<img>", ContentType::Html);
                    el.replace("<img>", ContentType::Text);

                    assert!(el.removed());
                },
            );

            assert_eq!(output, "<div>&lt;img&gt;</div>");

            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.replace("<img>", ContentType::Text);
                    el.replace("<img>", ContentType::Html);

                    assert!(el.removed());
                },
            );

            assert_eq!(output, "<div><img></div>");
        }
    });

    test("Remove", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element(
                "<div><span>Hi<inner-remove-me>Remove</inner-remove-me></span></div>",
                enc,
                "span",
                |el| {
                    el.prepend("<prepended>", ContentType::Html);
                    el.append("<appended>", ContentType::Html);
                    el.remove();

                    assert!(el.removed());
                },
            );

            assert_eq!(output, "<div></div>");
        }
    });

    test("Remove - unfinished end tag", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Heello</span  ", enc, "span", |el| {
                el.remove();

                assert!(el.removed());
            });

            assert_eq!(output, "<div>");
        }
    });

    test("Remove and keep content", {
        for enc in ASCII_COMPATIBLE_ENCODINGS.iter() {
            let output = rewrite_element("<div><span>Hi</span></div>", enc, "span", |el| {
                el.prepend("<prepended>", ContentType::Html);
                el.append("<appended>", ContentType::Html);
                el.remove_and_keep_content();

                assert!(el.removed());
            });

            assert_eq!(output, "<div><prepended>Hi<appended></div>");
        }
    });

    test("Multiple consequent removes", {
        let html = "<div><span>42</span></div><h1>Hello</h1><h2>Hello2</h2>";
        let mut output = Output::new(UTF_8);

        {
            let mut rewriter = HtmlRewriter::try_new(
                vec![
                    (
                        &"div".parse().unwrap(),
                        ElementContentHandlers::default().element(|el| {
                            el.replace("hey & ya", ContentType::Html);
                        }),
                    ),
                    (
                        &"h1".parse().unwrap(),
                        ElementContentHandlers::default().element(|el| {
                            el.remove();
                        }),
                    ),
                    (
                        &"h2".parse().unwrap(),
                        ElementContentHandlers::default().element(|el| {
                            el.remove_and_keep_content();
                        }),
                    ),
                ],
                vec![],
                "utf-8",
                |c: &[u8]| output.push(c),
            )
            .unwrap();

            rewriter.write(html.as_bytes()).unwrap();
            rewriter.end().unwrap();
        }

        let actual: String = output.into();

        assert_eq!(actual, "hey & yaHello2");
    });

    test("Void element", {
        let output = rewrite_element("<img><span>Hi</span></img>", UTF_8, "img", |el| {
            el.after("<!--after-->", ContentType::Html);
            el.set_tag_name("img-foo").unwrap();
        });

        assert_eq!(output, "<img-foo><!--after--><span>Hi</span></img>");
    });

    test("Self-closing element", {
        let output = rewrite_element("<svg><foo/>Hi</foo></svg>", UTF_8, "foo", |el| {
            el.after("<!--after-->", ContentType::Html);
            el.set_tag_name("bar").unwrap();
        });

        assert_eq!(output, "<svg><bar/><!--after-->Hi</foo></svg>");
    });
});
