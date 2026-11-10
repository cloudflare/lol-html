use lol_html::html_content::ContentType;
use lol_html::{HtmlRewriter, Settings, doc_text, element};

const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const MATHML_NAMESPACE: &str = "http://www.w3.org/1998/Math/MathML";

struct AdvisoryCase {
    name: &'static str,
    input: &'static str,
    expected_output: &'static str,
    elements: &'static [(&'static str, &'static str)],
}

impl AdvisoryCase {
    fn unchanged(
        name: &'static str,
        input: &'static str,
        elements: &'static [(&'static str, &'static str)],
    ) -> Self {
        Self {
            name,
            input,
            expected_output: input,
            elements,
        }
    }
}

fn assert_report_cases(cases: &[AdvisoryCase]) {
    for case in cases {
        let expected_elements = case
            .elements
            .iter()
            .map(|(name, namespace)| ((*name).to_owned(), (*namespace).to_owned()))
            .collect::<Vec<_>>();

        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 2, 3, 7, 16, 31, case.input.len()] {
                    let mut observed = Vec::new();
                    let mut output = Vec::new();
                    let mut settings = Settings::new()
                        .with_strict(strict)
                        .append_element_content_handler(element!("*", |element| {
                            observed.push((element.tag_name(), element.namespace_uri().to_owned()));
                            Ok(())
                        }))
                        .append_element_content_handler(element!("a[href]", |element| {
                            element.set_attribute("href", "REWRITTEN")?;
                            Ok(())
                        }))
                        .append_element_content_handler(element!("title", |element| {
                            element.set_inner_content("REPLACED", ContentType::Text);
                            Ok(())
                        }));

                    if capture_text {
                        settings = settings.append_document_content_handler(doc_text!(|_| Ok(())));
                    }

                    let mut rewriter =
                        HtmlRewriter::new(settings, |bytes: &[u8]| output.extend_from_slice(bytes));

                    case.input
                        .as_bytes()
                        .chunks(chunk_size)
                        .try_for_each(|chunk| rewriter.write(chunk))
                        .and_then(|()| rewriter.end())
                        .expect("advisory input must rewrite successfully");
                    assert_eq!(
                        output,
                        case.expected_output.as_bytes(),
                        "{}: strict={strict}, capture_text={capture_text}, chunk_size={chunk_size}",
                        case.name
                    );
                    assert_eq!(
                        observed, expected_elements,
                        "{}: strict={strict}, capture_text={capture_text}, chunk_size={chunk_size}",
                        case.name
                    );
                }
            }
        }
    }
}

fn assert_sanitized_input(input: &str) {
    for strict in [false, true] {
        for capture_text in [false, true] {
            for chunk_size in [1, 2, 3, 7, 16, 31, input.len()] {
                let mut image_count = 0;
                let mut output = Vec::new();
                let mut settings = Settings::new()
                    .with_strict(strict)
                    .append_element_content_handler(element!(
                        "script, iframe, object, embed",
                        |element| {
                            element.remove();
                            Ok(())
                        }
                    ))
                    .append_element_content_handler(element!("*", |element| {
                        if element.tag_name() == "img" {
                            image_count += 1;
                            assert_eq!(
                                element.namespace_uri(),
                                HTML_NAMESPACE,
                                "the reported image must be HTML"
                            );
                        }

                        let attributes = element
                            .attributes()
                            .iter()
                            .map(|attribute| attribute.name())
                            .filter(|name| name.to_ascii_lowercase().starts_with("on"))
                            .collect::<Vec<_>>();

                        for name in attributes {
                            element.remove_attribute(&name);
                        }

                        Ok(())
                    }));

                if capture_text {
                    settings = settings.append_document_content_handler(doc_text!(|_| Ok(())));
                }

                let mut rewriter =
                    HtmlRewriter::new(settings, |bytes: &[u8]| output.extend_from_slice(bytes));

                input
                    .as_bytes()
                    .chunks(chunk_size)
                    .try_for_each(|chunk| rewriter.write(chunk))
                    .and_then(|()| rewriter.end())
                    .expect("sanitizer input must rewrite successfully");

                let output = String::from_utf8(output).expect("the ASCII input must remain UTF-8");

                assert_eq!(
                    image_count, 1,
                    "the sanitizer must see the actual image: {input}"
                );
                assert!(
                    !output.contains("onerror"),
                    "the handler must remove the event attribute: {output}"
                );
                assert!(
                    output.contains("<img"),
                    "filtering must not discard the actual image"
                );
            }
        }
    }
}

fn observe_ids(
    input: &str,
    strict: bool,
    capture_text: bool,
    chunk_size: usize,
) -> Vec<(String, String)> {
    let mut observed = Vec::new();
    let mut output = Vec::new();
    let mut settings = Settings::new()
        .with_strict(strict)
        .append_element_content_handler(element!("[id]", |element| {
            observed.push((
                element
                    .get_attribute("id")
                    .expect("the selector requires an id"),
                element.namespace_uri().to_owned(),
            ));
            Ok(())
        }));

    if capture_text {
        settings = settings.append_document_content_handler(doc_text!(|_| Ok(())));
    }

    let mut rewriter = HtmlRewriter::new(settings, |bytes: &[u8]| output.extend_from_slice(bytes));

    input
        .as_bytes()
        .chunks(chunk_size)
        .try_for_each(|chunk| rewriter.write(chunk))
        .and_then(|()| rewriter.end())
        .expect("namespace control must rewrite successfully");
    assert_eq!(
        output,
        input.as_bytes(),
        "namespace observation must preserve the source"
    );
    observed
}

/// Self-closing roots must not expose pseudo-elements inside the following HTML text element.
#[test]
fn test_parser_namespace_advisories_self_closing_roots() {
    assert_report_cases(&[
        AdvisoryCase::unchanged(
            "A1",
            r#"<svg/><script>var s = "<a href='x'>";</script>"#,
            &[("svg", SVG_NAMESPACE), ("script", HTML_NAMESPACE)],
        ),
        AdvisoryCase::unchanged(
            "A2",
            r#"<math/><script>var s = "<a href='x'>";</script>"#,
            &[("math", MATHML_NAMESPACE), ("script", HTML_NAMESPACE)],
        ),
        AdvisoryCase::unchanged(
            "A3",
            r#"<svg><svg/></svg><script>var s = "<a href='x'>";</script>"#,
            &[
                ("svg", SVG_NAMESPACE),
                ("svg", SVG_NAMESPACE),
                ("script", HTML_NAMESPACE),
            ],
        ),
        AdvisoryCase::unchanged(
            "A4",
            r#"<svg/><script>var s = "<title>x</title>";</script>"#,
            &[("svg", SVG_NAMESPACE), ("script", HTML_NAMESPACE)],
        ),
        AdvisoryCase::unchanged(
            "A5",
            r#"<svg/><style>a[title="<a href='x'>"]{}</style>"#,
            &[("svg", SVG_NAMESPACE), ("style", HTML_NAMESPACE)],
        ),
        AdvisoryCase::unchanged(
            "ctrl-A",
            r#"<svg></svg><script>var s = "<a href='x'>";</script>"#,
            &[("svg", SVG_NAMESPACE), ("script", HTML_NAMESPACE)],
        ),
        AdvisoryCase::unchanged(
            "S1",
            r#"<svg/><script><!--</script><img src=x onerror=alert(1)>--></script>"#,
            &[
                ("svg", SVG_NAMESPACE),
                ("script", HTML_NAMESPACE),
                ("img", HTML_NAMESPACE),
            ],
        ),
        AdvisoryCase::unchanged(
            "ctrl-S",
            r#"<script><!--</script><img src=x onerror=alert(1)>--></script>"#,
            &[("script", HTML_NAMESPACE), ("img", HTML_NAMESPACE)],
        ),
    ]);
}

/// The sanitizer must see the real image even after comment-like text in an allowed text element.
#[test]
fn test_parser_namespace_advisories_self_closing_sanitizer() {
    for prefix in [
        "<p>hi</p>",
        "<style>a{}</style>",
        "<svg/><style><!--</style>",
        "<svg/><style><![CDATA[</style>",
        "<math/><style><!--</style>",
        "<svg/><title><!--</title>",
        "<svg/><textarea><!--</textarea>",
    ] {
        assert_sanitized_input(&format!("{prefix}<img src=x onerror=alert(1)>"));
    }

    assert_sanitized_input("<svg/><script><!--</script><img src=x onerror=alert(1)>--></script>");
}

/// Element namespace overrides must be consumed without popping enclosing foreign roots.
#[test]
fn test_parser_namespace_advisories_self_closing_namespace_lifecycle() {
    for (input, expected) in [
        (
            "<svg id=svg /><p id=after></p>",
            vec![("svg", SVG_NAMESPACE), ("after", HTML_NAMESPACE)],
        ),
        (
            "<math id=math /><p id=after></p>",
            vec![("math", MATHML_NAMESPACE), ("after", HTML_NAMESPACE)],
        ),
        (
            "<svg><svg id=inner /><g id=foreign></g></svg><p id=after></p>",
            vec![
                ("inner", SVG_NAMESPACE),
                ("foreign", SVG_NAMESPACE),
                ("after", HTML_NAMESPACE),
            ],
        ),
        (
            "<math><math id=inner /><mrow id=foreign></mrow></math><p id=after></p>",
            vec![
                ("inner", MATHML_NAMESPACE),
                ("foreign", MATHML_NAMESPACE),
                ("after", HTML_NAMESPACE),
            ],
        ),
        (
            "<svg id=first /><math id=second /><p id=after></p>",
            vec![
                ("first", SVG_NAMESPACE),
                ("second", MATHML_NAMESPACE),
                ("after", HTML_NAMESPACE),
            ],
        ),
        (
            "<svg id=ordinary/><g id=foreign></g></svg><p id=after></p>",
            vec![
                ("ordinary/", SVG_NAMESPACE),
                ("foreign", SVG_NAMESPACE),
                ("after", HTML_NAMESPACE),
            ],
        ),
    ] {
        let expected = expected
            .into_iter()
            .map(|(name, namespace)| (name.to_owned(), namespace.to_owned()))
            .collect::<Vec<_>>();

        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    assert_eq!(
                        observe_ids(input, strict, capture_text, chunk_size),
                        expected,
                        "self-closing namespace lifetimes must survive chunked handoffs: {input}"
                    );
                }
            }
        }
    }
}

/// Self-closing roots restore HTML CDATA rules without disabling genuine foreign CDATA.
#[test]
fn test_parser_namespace_advisories_self_closing_cdata() {
    for input in [
        "<svg/><![CDATA[><img id=real>]]>",
        "<math/><![CDATA[><img id=real>]]>",
        "<svg><![CDATA[<img id=phantom>]]></svg><img id=real>",
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    assert_eq!(
                        observe_ids(input, strict, capture_text, chunk_size),
                        vec![("real".to_owned(), HTML_NAMESPACE.to_owned())],
                        "CDATA permission must follow content, not the emitted element: {input}"
                    );
                }
            }
        }
    }
}

/// Namespace requests abandoned at EOF must not fabricate a completed start tag.
#[test]
fn test_parser_namespace_advisories_self_closing_incomplete_input() {
    for input in [
        "<svg",
        "<svg/",
        "<svg id=unfinished/",
        "<math id='unfinished",
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, input.len()] {
                    assert_eq!(
                        observe_ids(input, strict, capture_text, chunk_size),
                        Vec::new(),
                        "incomplete roots must remain unreported: {input}"
                    );
                }
            }
        }
    }
}

/// Scanner-only rewriting and document text handlers must agree on the HTML text context.
#[test]
fn test_parser_namespace_advisories_self_closing_without_element_handlers() {
    for (input, expected_text) in [
        (
            r#"<svg/><script>var s = "<a href='x'>";</script>"#,
            r#"var s = "<a href='x'>";"#,
        ),
        (
            r#"<math/><style>a[title="<a href='x'>"]{}</style>"#,
            r#"a[title="<a href='x'>"]{}"#,
        ),
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    let mut text = String::new();
                    let mut output = Vec::new();
                    let mut settings = Settings::new().with_strict(strict);

                    if capture_text {
                        settings = settings.append_document_content_handler(doc_text!(|chunk| {
                            text.push_str(chunk.as_str());
                            Ok(())
                        }));
                    }

                    let mut rewriter =
                        HtmlRewriter::new(settings, |bytes: &[u8]| output.extend_from_slice(bytes));

                    input
                        .as_bytes()
                        .chunks(chunk_size)
                        .try_for_each(|chunk| rewriter.write(chunk))
                        .and_then(|()| rewriter.end())
                        .expect("no-element-handler input must rewrite successfully");
                    assert_eq!(
                        output,
                        input.as_bytes(),
                        "the rewrite must preserve source bytes"
                    );
                    if capture_text {
                        assert_eq!(
                            text, expected_text,
                            "pseudo-elements must remain part of the text"
                        );
                    }
                }
            }
        }
    }
}

/// HTML breakouts must clear every foreign frame left by the original nested-root examples.
#[test]
fn test_parser_namespace_advisories_foreign_breakouts() {
    assert_report_cases(&[
        AdvisoryCase::unchanged(
            "B1",
            r#"<svg><svg><div><script>var s = "<a href='x'>";</script>"#,
            &[
                ("svg", SVG_NAMESPACE),
                ("svg", SVG_NAMESPACE),
                ("div", HTML_NAMESPACE),
                ("script", HTML_NAMESPACE),
            ],
        ),
        AdvisoryCase::unchanged(
            "B2",
            r#"<svg><svg></p><script>var s = "<a href='x'>";</script>"#,
            &[
                ("svg", SVG_NAMESPACE),
                ("svg", SVG_NAMESPACE),
                ("script", HTML_NAMESPACE),
            ],
        ),
        AdvisoryCase::unchanged(
            "ctrl-B",
            r#"<svg><div><script>var s = "<a href='x'>";</script>"#,
            &[
                ("svg", SVG_NAMESPACE),
                ("div", HTML_NAMESPACE),
                ("script", HTML_NAMESPACE),
            ],
        ),
    ]);
}

/// Breakouts stop at integration points, while ordinary foreign ends and unqualified fonts do not.
#[test]
fn test_parser_namespace_advisories_foreign_breakout_controls() {
    let mut cases = vec![
        (
            "<svg><svg></svg><g id=foreign></g></svg><p id=after></p>".to_owned(),
            vec![("foreign", SVG_NAMESPACE), ("after", HTML_NAMESPACE)],
        ),
        (
            concat!(
                "<svg><svg><font id=foreign></font>",
                "<g id=still-foreign></g></svg></svg><p id=after></p>"
            )
            .to_owned(),
            vec![
                ("foreign", SVG_NAMESPACE),
                ("still-foreign", SVG_NAMESPACE),
                ("after", HTML_NAMESPACE),
            ],
        ),
        (
            concat!(
                "<svg><desc><svg><svg><div id=html></div>",
                r#"<script id=script>var s = "<img id=phantom>";</script>"#,
                "</desc><g id=foreign></g></svg><p id=after></p>"
            )
            .to_owned(),
            vec![
                ("html", HTML_NAMESPACE),
                ("script", HTML_NAMESPACE),
                ("foreign", SVG_NAMESPACE),
                ("after", HTML_NAMESPACE),
            ],
        ),
        (
            concat!(
                "<math><mtext><svg><svg><div id=html></div>",
                r#"<script id=script>var s = "<img id=phantom>";</script>"#,
                "</mtext><mrow id=foreign></mrow></math><p id=after></p>"
            )
            .to_owned(),
            vec![
                ("html", HTML_NAMESPACE),
                ("script", HTML_NAMESPACE),
                ("foreign", MATHML_NAMESPACE),
                ("after", HTML_NAMESPACE),
            ],
        ),
        (
            "<svg><svg><div><![CDATA[><img id=real>]]>".to_owned(),
            vec![("real", HTML_NAMESPACE)],
        ),
    ];

    cases.extend(
        ["div", "span", "font color", "font face='x'", "font size=3"]
            .into_iter()
            .map(|start| {
                (
                    format!(
                        concat!(
                            "<svg><svg><{} id=html>",
                            r#"<script id=script>var s = "<img id=phantom>";</script>"#,
                        ),
                        start
                    ),
                    vec![("html", HTML_NAMESPACE), ("script", HTML_NAMESPACE)],
                )
            }),
    );
    cases.extend(["</p>", "</br>"].into_iter().map(|end| {
        (
            format!(r#"<math><math>{end}<script id=script>var s = "<img id=phantom>";</script>"#),
            vec![("script", HTML_NAMESPACE)],
        )
    }));

    for (input, expected) in cases {
        let expected = expected
            .into_iter()
            .map(|(name, namespace)| (name.to_owned(), namespace.to_owned()))
            .collect::<Vec<_>>();

        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    assert_eq!(
                        observe_ids(&input, strict, capture_text, chunk_size),
                        expected,
                        "foreign breakout boundaries must preserve the correct context: {input}"
                    );
                }
            }
        }
    }
}

/// The nested-foreign sanitizer example must expose the image after the HTML style element.
#[test]
fn test_parser_namespace_advisories_foreign_breakout_sanitizer() {
    assert_sanitized_input("<svg><svg><div><style><!--</style><img src=x onerror=alert(1)>");
}
