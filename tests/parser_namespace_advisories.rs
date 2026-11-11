use lol_html::errors::RewritingError;
use lol_html::html_content::ContentType;
use lol_html::{HtmlRewriter, MemorySettings, Settings, bail_out, doc_text, element};
use std::cell::Cell;

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
    observe_id_chunks(
        input,
        strict,
        capture_text,
        input.as_bytes().chunks(chunk_size),
    )
}

fn observe_id_chunks<'input>(
    input: &'input str,
    strict: bool,
    capture_text: bool,
    chunks: impl IntoIterator<Item = &'input [u8]>,
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

    chunks
        .into_iter()
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

/// The real HTML title is rewritten, but its end tag must not expose the following script text.
#[test]
fn test_parser_namespace_advisories_integration_point_reported_rewriting() {
    assert_report_cases(&[
        AdvisoryCase {
            name: "D1",
            input: concat!(
                "<svg><desc><title>t</title>",
                r#"<script>var s = "<a href='x'>";</script></desc></svg>"#,
            ),
            expected_output: concat!(
                "<svg><desc><title>REPLACED</title>",
                r#"<script>var s = "<a href='x'>";</script></desc></svg>"#,
            ),
            elements: &[
                ("svg", SVG_NAMESPACE),
                ("desc", SVG_NAMESPACE),
                ("title", HTML_NAMESPACE),
                ("script", HTML_NAMESPACE),
            ],
        },
        AdvisoryCase::unchanged(
            "ctrl-D",
            r#"<svg><desc><script>var s = "<a href='x'>";</script></desc></svg>"#,
            &[
                ("svg", SVG_NAMESPACE),
                ("desc", SVG_NAMESPACE),
                ("script", HTML_NAMESPACE),
            ],
        ),
    ]);
}

/// The original phantom-image observations and insertion exploit must never reach an image handler.
#[test]
fn test_parser_namespace_advisories_integration_point_reported_image_mutation() {
    for input in [
        concat!(
            "<svg><foreignObject><desc></desc>",
            "<script>ok();<img id=bridge></script></foreignObject></svg>",
        ),
        concat!(
            "<svg><foreignObject><desc></desc>",
            "<script>x<img id=t></script></foreignObject></svg>",
        ),
        "<svg><desc><desc></desc><script>x<img id=t></script></desc></svg>",
        concat!(
            "<svg><foreignObject><title></title>",
            "<script>x<img id=t></script></foreignObject></svg>",
        ),
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 2, 3, 7, 16, 31, input.len()] {
                    let mut image_count = 0;
                    let mut output = Vec::new();
                    let mut settings = Settings::new()
                        .with_strict(strict)
                        .append_element_content_handler(element!("img", |_| {
                            image_count += 1;
                            Ok(())
                        }))
                        .append_element_content_handler(element!("img#t", |element| {
                            element
                                .before("</script><img src=x onerror=alert(1)>", ContentType::Html);
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
                        .expect("the original integration-point reports must rewrite successfully");
                    assert_eq!(
                        image_count, 0,
                        "script text must not match image handlers: {input}"
                    );
                    assert_eq!(
                        output,
                        input.as_bytes(),
                        "phantom insertion must not run: {input}"
                    );
                }
            }
        }
    }
}

/// Every integration-point opener must survive unrelated and same-named HTML descendant closures.
#[test]
fn test_parser_namespace_advisories_integration_point_lifetimes() {
    for (root, opener, namespace) in [
        ("svg", "desc", SVG_NAMESPACE),
        ("svg", "title", SVG_NAMESPACE),
        ("svg", "foreignObject", SVG_NAMESPACE),
        ("math", "mi", MATHML_NAMESPACE),
        ("math", "mo", MATHML_NAMESPACE),
        ("math", "mn", MATHML_NAMESPACE),
        ("math", "ms", MATHML_NAMESPACE),
        ("math", "mtext", MATHML_NAMESPACE),
        ("math", "annotation-xml", MATHML_NAMESPACE),
    ] {
        let attributes = match opener {
            "annotation-xml" => " encoding='text/html'",
            _ => "",
        };

        for descendant in [
            "desc",
            "title",
            "foreignObject",
            "mi",
            "mtext",
            "annotation-xml",
        ] {
            let descendant_text = match descendant {
                "title" => "<img id=title-phantom>",
                _ => "",
            };
            let input = format!(
                concat!(
                    "<{root}><{opener}{attributes} id=opener>",
                    "<{descendant} id=descendant>{descendant_text}</{descendant}>",
                    r#"<script id=script>var s = "<img id=phantom>";</script>"#,
                    "</{opener}><g id=foreign></g></{root}><p id=after></p>"
                ),
                root = root,
                opener = opener,
                attributes = attributes,
                descendant = descendant,
                descendant_text = descendant_text,
            );
            let expected = [
                ("opener", namespace),
                ("descendant", HTML_NAMESPACE),
                ("script", HTML_NAMESPACE),
                ("foreign", namespace),
                ("after", HTML_NAMESPACE),
            ]
            .map(|(name, namespace)| (name.to_owned(), namespace.to_owned()));

            for strict in [false, true] {
                for capture_text in [false, true] {
                    for chunk_size in [1, 3, 7, input.len()] {
                        assert_eq!(
                            observe_ids(&input, strict, capture_text, chunk_size),
                            expected,
                            "only the actual integration-point closer may end HTML context: {input}"
                        );
                    }
                }
            }
        }
    }
}

/// Same-named descendants, including ignored HTML self-closing flags, can nest beyond one level.
#[test]
fn test_parser_namespace_advisories_integration_point_nested_same_name() {
    for (root, opener, namespace) in [
        ("svg", "desc", SVG_NAMESPACE),
        ("svg", "foreignObject", SVG_NAMESPACE),
        ("math", "mtext", MATHML_NAMESPACE),
        ("math", "annotation-xml", MATHML_NAMESPACE),
    ] {
        let attributes = match opener {
            "annotation-xml" => " encoding='application/xhtml+xml'",
            _ => "",
        };

        for depth in [1, 2, 9, 33] {
            let input = format!(
                concat!(
                    "<{root}><{opener}{attributes}>{descendants}{closers}",
                    r#"<script id=html>"<img id=phantom>"</script>"#,
                    "</{opener}><g id=foreign></g></{root}><p id=after></p>"
                ),
                root = root,
                opener = opener,
                attributes = attributes,
                descendants = format!("<{opener}/>").repeat(depth),
                closers = format!("</{opener}>").repeat(depth),
            );
            let expected = [
                ("html", HTML_NAMESPACE),
                ("foreign", namespace),
                ("after", HTML_NAMESPACE),
            ]
            .map(|(name, namespace)| (name.to_owned(), namespace.to_owned()));

            for strict in [false, true] {
                for capture_text in [false, true] {
                    for chunk_size in [1, 3, 7, input.len()] {
                        assert_eq!(
                            observe_ids(&input, strict, capture_text, chunk_size),
                            expected,
                            "each same-named HTML descendant consumes only its own closer: {input}"
                        );
                    }
                }
            }
        }
    }
}

/// Nested foreign scopes must suspend and restore the outer integration-point occurrences.
#[test]
fn test_parser_namespace_advisories_integration_point_nested_foreign() {
    for input in [
        concat!(
            "<svg><desc><desc><svg><desc><desc></desc></desc></svg></desc>",
            r#"<script id=html>"<img id=phantom>"</script>"#,
            "</desc><g id=foreign></g></svg><p id=after></p>",
        ),
        concat!(
            "<svg><desc><desc><svg><svg><div></div></desc>",
            r#"<script id=html>"<img id=phantom>"</script>"#,
            "</desc><g id=foreign></g></svg><p id=after></p>",
        ),
        concat!(
            "<svg><foreignObject><foreignObject><math><mtext><mtext></mtext></mtext></math>",
            "</foreignObject>",
            r#"<script id=html>"<img id=phantom>"</script>"#,
            "</foreignObject><g id=foreign></g></svg><p id=after></p>",
        ),
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    assert_eq!(
                        observe_ids(input, strict, capture_text, chunk_size),
                        vec![
                            ("html".to_owned(), HTML_NAMESPACE.to_owned()),
                            ("foreign".to_owned(), SVG_NAMESPACE.to_owned()),
                            ("after".to_owned(), HTML_NAMESPACE.to_owned()),
                        ],
                        "the inner scope must not consume outer lifetime metadata: {input}"
                    );
                }
            }
        }
    }
}

/// Empty compact hashes must not conflate annotation-xml with unrelated custom element names.
#[test]
fn test_parser_namespace_advisories_integration_point_annotation_identity() {
    for input in [
        concat!(
            "<math><annotation-xml encoding='text/html'><custom-element></custom-element>",
            r#"<script id=html>"<img id=phantom>"</script>"#,
            "</annotation-xml><mrow id=foreign></mrow></math>",
        ),
        concat!(
            "<math><annotation-xml encoding='text/html'></other-custom>",
            r#"<script id=html>"<img id=phantom>"</script>"#,
            "</annotation-xml><mrow id=foreign></mrow></math>",
        ),
        concat!(
            "<math><annotation-xml encoding='TeXt/HtMl'><ANNOTATION-XML></annotation-XML>",
            r#"<script id=html>"<img id=phantom>"</script>"#,
            "</annotation-xml><mrow id=foreign></mrow></math>",
        ),
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    assert_eq!(
                        observe_ids(input, strict, capture_text, chunk_size),
                        vec![
                            ("html".to_owned(), HTML_NAMESPACE.to_owned()),
                            ("foreign".to_owned(), MATHML_NAMESPACE.to_owned()),
                        ],
                        "annotation-xml lifetime checks require the full literal name: {input}"
                    );
                }
            }
        }
    }
}

/// Self-closing foreign integration points and non-HTML annotations must not open an HTML frame.
#[test]
fn test_parser_namespace_advisories_integration_point_entry_controls() {
    for (input, namespace) in [
        (
            "<svg><desc id=point /><g id=foreign></g></svg>",
            SVG_NAMESPACE,
        ),
        (
            "<svg><title id=point /><g id=foreign></g></svg>",
            SVG_NAMESPACE,
        ),
        (
            "<svg><foreignObject id=point /><g id=foreign></g></svg>",
            SVG_NAMESPACE,
        ),
        (
            "<math><mtext id=point /><mrow id=foreign></mrow></math>",
            MATHML_NAMESPACE,
        ),
        (
            concat!(
                "<math><annotation-xml encoding='text/html' id=point />",
                "<mrow id=foreign></mrow></math>"
            ),
            MATHML_NAMESPACE,
        ),
        (
            "<math><annotation-xml id=point><mrow id=foreign></mrow></annotation-xml></math>",
            MATHML_NAMESPACE,
        ),
        (
            concat!(
                "<math><annotation-xml encoding='text/plain' id=point>",
                "<mrow id=foreign></mrow></annotation-xml></math>"
            ),
            MATHML_NAMESPACE,
        ),
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    assert_eq!(
                        observe_ids(input, strict, capture_text, chunk_size),
                        vec![
                            ("point".to_owned(), namespace.to_owned()),
                            ("foreign".to_owned(), namespace.to_owned()),
                        ],
                        "only non-self-closing integration points open an HTML context: {input}"
                    );
                }
            }
        }
    }
}

/// All text modes named in the first report must keep their pseudo-images away from handlers.
#[test]
fn test_parser_namespace_advisories_integration_point_text_modes() {
    for text_element in ["script", "style", "xmp", "textarea", "noscript"] {
        let input = format!(
            concat!(
                "<svg><foreignObject><desc></desc>",
                "<{text_element}>x<img id=phantom></{text_element}>",
                "</foreignObject></svg><img id=real>"
            ),
            text_element = text_element,
        );

        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    assert_eq!(
                        observe_ids(&input, strict, capture_text, chunk_size),
                        vec![("real".to_owned(), HTML_NAMESPACE.to_owned())],
                        "the integration point must preserve {text_element} text parsing"
                    );
                }
            }
        }
    }
}

/// New lifetime metadata must fail through the existing memory error, including graceful output.
#[test]
fn test_parser_namespace_advisories_integration_point_memory_limit() {
    for input in [
        format!("<svg><foreignObject>{}", "<foreignObject>".repeat(1024)),
        format!(
            "<math><annotation-xml encoding='text/html'>{}",
            "<annotation-xml>".repeat(1024)
        ),
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for graceful in [false, true] {
                    for chunk_size in [1, 7, 257, input.len()] {
                        let mut output = Vec::new();
                        let mut fed_bytes = 0;
                        let bailed_out = Cell::new(false);
                        let mut settings = Settings::new()
                            .with_strict(strict)
                            .with_memory_settings(
                                MemorySettings::new()
                                    .with_preallocated_parsing_buffer_size(0)
                                    .with_max_allowed_memory_usage(4096)
                                    .with_graceful_bail_out_on_memory_limit_exceeded(graceful),
                            )
                            .append_bail_out_handler(bail_out!(|error, _| {
                                assert!(
                                    matches!(error, RewritingError::MemoryLimitExceeded(_)),
                                    "metadata exhaustion must use the existing memory error"
                                );
                                bailed_out.set(true);
                            }));

                        if capture_text {
                            settings =
                                settings.append_document_content_handler(doc_text!(|_| Ok(())));
                        }

                        let mut rewriter = HtmlRewriter::new(settings, |bytes: &[u8]| {
                            output.extend_from_slice(bytes);
                        });
                        let result = input.as_bytes().chunks(chunk_size).try_for_each(|chunk| {
                            fed_bytes += chunk.len();
                            rewriter.write(chunk)
                        });

                        assert!(
                            matches!(result, Err(RewritingError::MemoryLimitExceeded(_))),
                            "lifetime metadata must share the configured memory limit"
                        );
                        drop(rewriter);
                        assert_eq!(
                            bailed_out.get(),
                            graceful,
                            "bail-out handlers must honor settings"
                        );
                        if graceful {
                            output.extend_from_slice(&input.as_bytes()[fed_bytes..]);
                            assert_eq!(
                                output,
                                input.as_bytes(),
                                "graceful bail-out must not lose bytes"
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Any two-chunk boundary must preserve pending integration feedback and full-name checks.
#[test]
fn test_parser_namespace_advisories_integration_point_feedback_boundaries() {
    for (input, namespace) in [
        (
            concat!(
                "<svg><desc><desc></desc><script>x<img id=t></script></desc>",
                "<g id=foreign></g></svg><p id=after></p>"
            ),
            SVG_NAMESPACE,
        ),
        (
            concat!(
                "<math><annotation-xml encoding='text/html'><annotation-xml></annotation-xml>",
                "<script>x<img id=t></script></annotation-xml>",
                "<mrow id=foreign></mrow></math><p id=after></p>"
            ),
            MATHML_NAMESPACE,
        ),
    ] {
        let expected = [("foreign", namespace), ("after", HTML_NAMESPACE)]
            .map(|(name, namespace)| (name.to_owned(), namespace.to_owned()));

        for strict in [false, true] {
            for capture_text in [false, true] {
                for split in 0..=input.len() {
                    let (first, second) = input.as_bytes().split_at(split);

                    assert_eq!(
                        observe_id_chunks(input, strict, capture_text, [first, second]),
                        expected,
                        "integration feedback must survive split {split}: {input}"
                    );
                }
            }
        }
    }
}

/// The new memory failure must keep the documented fatal-error behavior on rewriter reuse.
#[test]
fn test_parser_namespace_advisories_integration_point_terminal_memory_error() {
    let input = format!("<svg><foreignObject>{}", "<foreignObject>".repeat(1024));

    for finalize in [false, true] {
        let settings = Settings::new().with_memory_settings(
            MemorySettings::new()
                .with_preallocated_parsing_buffer_size(0)
                .with_max_allowed_memory_usage(4096),
        );
        let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});
        let error = rewriter
            .write(input.as_bytes())
            .expect_err("metadata must exhaust the limit");

        assert!(
            matches!(error, RewritingError::MemoryLimitExceeded(_)),
            "metadata exhaustion must return the existing terminal memory error"
        );

        let reuse =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || match finalize {
                true => rewriter.end(),
                false => rewriter.write(b"more input"),
            }));

        assert!(
            reuse.is_err(),
            "only explicit reuse after the fatal error should panic"
        );
    }
}

/// Explicit ancestor closes must discard descendant records before the real integration close.
#[test]
fn test_parser_namespace_advisories_integration_point_ancestor_closure() {
    for (root, opener, namespace) in [
        ("svg", "desc", SVG_NAMESPACE),
        ("svg", "title", SVG_NAMESPACE),
        ("svg", "foreignObject", SVG_NAMESPACE),
        ("math", "mi", MATHML_NAMESPACE),
        ("math", "mo", MATHML_NAMESPACE),
        ("math", "mn", MATHML_NAMESPACE),
        ("math", "ms", MATHML_NAMESPACE),
        ("math", "mtext", MATHML_NAMESPACE),
        ("math", "annotation-xml", MATHML_NAMESPACE),
    ] {
        let attributes = match opener {
            "annotation-xml" => " encoding='text/html'",
            _ => "",
        };
        let descendant = match opener {
            "title" => "desc",
            _ => opener,
        };

        for ancestor in [
            "span",
            "div",
            "section",
            "custom-element",
            "abcdefghijklmnop",
        ] {
            let input = format!(
                concat!(
                    "<{root}><{opener}{attributes}><{ancestor}><{descendant}>",
                    "</{ancestor}>",
                    r#"<script id=html>"<img id=phantom>"</script>"#,
                    "</{opener}><g id=foreign></g></{root}><p id=after></p>"
                ),
                root = root,
                opener = opener,
                attributes = attributes,
                ancestor = ancestor,
                descendant = descendant,
            );
            let expected = [
                ("html", HTML_NAMESPACE),
                ("foreign", namespace),
                ("after", HTML_NAMESPACE),
            ]
            .map(|(identifier, namespace)| (identifier.to_owned(), namespace.to_owned()));

            for strict in [false, true] {
                for capture_text in [false, true] {
                    for chunk_size in [1, 2, 3, 7, 31, input.len()] {
                        assert_eq!(
                            observe_ids(&input, strict, capture_text, chunk_size),
                            expected,
                            "an explicit ancestor close must remove its descendant metadata: {input}"
                        );
                    }
                }
            }
        }
    }
}

/// A stale HTML frame must not hide an actual breakout image inside a foreign script element.
#[test]
fn test_parser_namespace_advisories_integration_point_ancestor_real_image() {
    let input = "<svg><desc><span><desc></span></desc><script><img id=real onerror=x></script>";

    for strict in [false, true] {
        for capture_text in [false, true] {
            for chunk_size in [1, 2, 3, 7, 31, input.len()] {
                let mut images = 0;
                let mut output = Vec::new();
                let mut settings = Settings::new()
                    .with_strict(strict)
                    .append_element_content_handler(element!("img", |element| {
                        images += 1;
                        assert_eq!(
                            element.namespace_uri(),
                            HTML_NAMESPACE,
                            "the real image is HTML"
                        );
                        element.remove_attribute("onerror");
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
                    .expect("the real image must remain rewritable");
                assert_eq!(
                    images, 1,
                    "the ancestor close must expose the actual image to filtering"
                );
                assert_eq!(
                    output, b"<svg><desc><span><desc></span></desc><script><img id=real></script>",
                    "filtering must remove the real image's event attribute"
                );
            }
        }
    }
}

/// Exact names, unmatched closes, HTML voids, and ignored self-closing flags must stay distinct.
#[test]
fn test_parser_namespace_advisories_integration_point_ancestor_identity_controls() {
    for input in [
        "<svg><desc><span/><desc></span></desc><g id=foreign></g></svg>",
        "<svg><desc><CUSTOM-ELEMENT><desc></custom-element></desc><g id=foreign></g></svg>",
        "<svg><desc><abcdefghijklmnop><desc></ABCDEFGHIJKLMNOP></desc><g id=foreign></g></svg>",
        concat!(
            "<svg><desc><first-custom><desc></other-custom></desc>",
            r#"<script id=html>"<img id=phantom>"</script>"#,
            "</first-custom></desc><g id=foreign></g></svg>",
        ),
        concat!(
            "<svg><desc><img><desc></img></desc>",
            r#"<script id=html>"<img id=phantom>"</script>"#,
            "</desc><g id=foreign></g></svg>",
        ),
        concat!(
            "<svg><desc><span><div><desc></span></desc>",
            r#"<script id=html>"<img id=phantom>"</script>"#,
            "</div></span></desc><g id=foreign></g></svg>",
        ),
        "<svg><desc><span><desc></span></desc><g id=foreign></g></svg>",
        "<svg><desc><div><p><desc></div></desc><g id=foreign></g></svg>",
    ] {
        let expected = if input.contains("id=html") {
            vec![
                ("html".to_owned(), HTML_NAMESPACE.to_owned()),
                ("foreign".to_owned(), SVG_NAMESPACE.to_owned()),
            ]
        } else {
            vec![("foreign".to_owned(), SVG_NAMESPACE.to_owned())]
        };

        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    assert_eq!(
                        observe_ids(input, strict, capture_text, chunk_size),
                        expected,
                        "only a real non-void ancestor close may discard descendant metadata: {input}"
                    );
                }
            }
        }
    }
}

/// Entering a nested foreign scope or breaking out of it must preserve the enclosing ancestry.
#[test]
fn test_parser_namespace_advisories_integration_point_ancestor_foreign_scopes() {
    for input in [
        concat!(
            "<svg><desc><span><desc><svg><desc><span><desc></span></desc></svg>",
            "</span></desc><g id=foreign></g></svg>",
        ),
        "<svg><desc><svg><span><desc></span></desc><g id=foreign></g></svg>",
        concat!(
            "<svg><desc><svg><font color=x><desc></font></desc>",
            "<g id=foreign></g></svg>",
        ),
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for split in 0..=input.len() {
                    let (first, second) = input.as_bytes().split_at(split);

                    assert_eq!(
                        observe_id_chunks(input, strict, capture_text, [first, second]),
                        vec![("foreign".to_owned(), SVG_NAMESPACE.to_owned())],
                        "foreign scopes must not lose or consume the outer ancestry: split={split}, {input}"
                    );
                }
            }
        }
    }
}

/// A foreign opener cannot close through an HTML descendant that has not itself been closed.
#[test]
fn test_parser_namespace_advisories_integration_point_ancestor_unclosed_controls() {
    for input in [
        "<svg><desc><span></desc><g id=html></g></span></desc><g id=foreign></g></svg>",
        concat!(
            "<svg><title><custom-element><desc></other-custom></title><g id=html></g>",
            "</custom-element></title><g id=foreign></g></svg>",
        ),
        concat!(
            "<math><annotation-xml encoding='text/html'><span></annotation-xml><g id=html></g>",
            "</span></annotation-xml><mrow id=foreign></mrow></math>",
        ),
    ] {
        let namespace = if input.starts_with("<math>") {
            MATHML_NAMESPACE
        } else {
            SVG_NAMESPACE
        };
        let expected = [("html", HTML_NAMESPACE), ("foreign", namespace)]
            .map(|(identifier, namespace)| (identifier.to_owned(), namespace.to_owned()));

        for strict in [false, true] {
            for capture_text in [false, true] {
                for chunk_size in [1, 3, 7, input.len()] {
                    assert_eq!(
                        observe_ids(input, strict, capture_text, chunk_size),
                        expected,
                        "a foreign end tag must not close through an HTML descendant: {input}"
                    );
                }
            }
        }
    }
}

/// CDATA permission must change only when the actual foreign integration opener closes.
#[test]
fn test_parser_namespace_advisories_integration_point_ancestor_cdata() {
    for input in [
        concat!(
            "<svg><desc><span><desc></span></desc>",
            "<![CDATA[><img id=phantom>]]></svg><img id=real>",
        ),
        concat!(
            "<svg><desc><span><desc></span>",
            "<span><![CDATA[><img id=real>]]></span></desc></svg>",
        ),
    ] {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for split in 0..=input.len() {
                    let (first, second) = input.as_bytes().split_at(split);

                    assert_eq!(
                        observe_id_chunks(input, strict, capture_text, [first, second]),
                        vec![("real".to_owned(), HTML_NAMESPACE.to_owned())],
                        "CDATA must follow the actual integration lifetime: split={split}, {input}"
                    );
                }
            }
        }
    }
}

/// Distinct ancestor names and retained literal bytes must share the existing memory failure path.
#[test]
fn test_parser_namespace_advisories_integration_point_ancestor_memory_limit() {
    let ancestors = [
        (0..512)
            .map(|index| {
                format!(
                    "<ancestor{}{}>",
                    char::from(b'a' + (index % 26) as u8),
                    char::from(b'a' + (index / 26) as u8)
                )
            })
            .collect::<String>(),
        format!("<{}>", "custom-element-".repeat(512)),
    ];

    for (input, memory_limit) in std::iter::once(("<svg><desc>".to_owned(), 0)).chain(
        ancestors
            .into_iter()
            .map(|ancestors| (format!("<svg><desc>{ancestors}"), 4096)),
    ) {
        for strict in [false, true] {
            for capture_text in [false, true] {
                for graceful in [false, true] {
                    for chunk_size in [1, 7, 257, input.len()] {
                        let bailed_out = Cell::new(false);
                        let mut output = Vec::new();
                        let mut fed_bytes = 0;
                        let mut settings = Settings::new()
                            .with_strict(strict)
                            .with_memory_settings(
                                MemorySettings::new()
                                    .with_preallocated_parsing_buffer_size(0)
                                    .with_max_allowed_memory_usage(memory_limit)
                                    .with_graceful_bail_out_on_memory_limit_exceeded(graceful),
                            )
                            .append_bail_out_handler(bail_out!(|error, _| {
                                assert!(
                                    matches!(error, RewritingError::MemoryLimitExceeded(_)),
                                    "ancestry exhaustion must preserve the existing error type"
                                );
                                bailed_out.set(true);
                            }));

                        if capture_text {
                            settings =
                                settings.append_document_content_handler(doc_text!(|_| Ok(())));
                        }

                        let mut rewriter = HtmlRewriter::new(settings, |bytes: &[u8]| {
                            output.extend_from_slice(bytes)
                        });
                        let result = input.as_bytes().chunks(chunk_size).try_for_each(|chunk| {
                            fed_bytes += chunk.len();
                            rewriter.write(chunk)
                        });

                        assert!(
                            matches!(result, Err(RewritingError::MemoryLimitExceeded(_))),
                            "ancestor records, name index, and literal names must respect the shared limit"
                        );
                        drop(rewriter);
                        assert_eq!(
                            bailed_out.get(),
                            graceful,
                            "ancestry failures must honor graceful bail-out settings"
                        );
                        if graceful {
                            output.extend_from_slice(&input.as_bytes()[fed_bytes..]);
                            assert_eq!(
                                output,
                                input.as_bytes(),
                                "ancestry failures must not lose source bytes"
                            );
                        }
                    }
                }
            }
        }
    }

    let input = format!(
        "<svg><desc><custom-element{}><desc></custom-element></desc></svg>",
        " a".repeat(4096)
    );

    for strict in [false, true] {
        for chunk_size in [1, 7, input.len()] {
            let mut output = Vec::new();
            let settings = Settings::new().with_strict(strict).with_memory_settings(
                MemorySettings::new()
                    .with_preallocated_parsing_buffer_size(0)
                    .with_max_allowed_memory_usage(4096),
            );
            let mut rewriter =
                HtmlRewriter::new(settings, |bytes: &[u8]| output.extend_from_slice(bytes));

            input
                .as_bytes()
                .chunks(chunk_size)
                .try_for_each(|chunk| rewriter.write(chunk))
                .and_then(|()| rewriter.end())
                .expect("ancestor-name tracking must not force attribute lexing");
            assert_eq!(
                output,
                input.as_bytes(),
                "scanner-only custom ancestors must preserve source bytes"
            );
        }
    }
}
