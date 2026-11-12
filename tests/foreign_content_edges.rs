//! Edge cases where a namespace and a parsing mode are easy to confuse.
//!
//! The HTML parser has no "XML mode": each of these behaviours switches on a
//! different quantity, and they do not coincide.
//!
//! * The element namespace is the DOM namespace (`Namespace`), reported by
//!   `Element::namespace_uri()`.
//! * The text type of an element's content (`ScriptData`/`RAWTEXT`/`RCDATA`)
//!   is chosen by the insertion mode plus the tag name, not by the namespace
//!   of the element: `<svg><script>` is a foreign element whose content is
//!   ordinary markup, while an HTML `<script>` switches the tokenizer.
//! * CDATA permission follows the *adjusted current node's* namespace, so a
//!   CDATA section is text inside an SVG `<desc>` (an HTML integration point)
//!   but a bogus comment inside an HTML `<p>` nested in that `<desc>`.
//! * The self-closing flag is acknowledged in foreign content and ignored in
//!   HTML content, so `<svg><script/>` closes immediately while
//!   `<svg><desc><script/>` does not.

use std::cell::RefCell;
use std::rc::Rc;

use lol_html::{RewriteStrSettings, comments, element, rewrite_str, text};

const HTML: &str = "http://www.w3.org/1999/xhtml";
const SVG: &str = "http://www.w3.org/2000/svg";
const MATHML: &str = "http://www.w3.org/1998/Math/MathML";

/// Rewrites `input` with a single handler on `selector`, returning the output
/// and the `(tag_name, namespace_uri)` of every element the handler matched.
fn collect(input: &str, selector: &str) -> (String, Vec<(String, &'static str)>) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        selector,
        move |el| {
            sink.borrow_mut().push((el.tag_name(), el.namespace_uri()));
            Ok(())
        }
    ));
    let output = rewrite_str(input, settings).unwrap();
    let seen = seen.borrow().clone();
    (output, seen)
}

fn expected(elements: &[(&str, &'static str)]) -> Vec<(String, &'static str)> {
    elements
        .iter()
        .map(|(name, ns)| ((*name).to_string(), *ns))
        .collect()
}

/// Joined text content the rewriter reports for `input`.
///
/// Text events are chunked arbitrarily, so chunks are concatenated before
/// comparison; empty chunks are dropped by the concatenation itself.
fn text_in(input: &str) -> String {
    let seen = Rc::new(RefCell::new(String::new()));
    let sink = Rc::clone(&seen);
    let settings = RewriteStrSettings::new().append_element_content_handler(text!(
        "*",
        move |chunk| {
            sink.borrow_mut().push_str(chunk.as_str());
            Ok(())
        }
    ));
    let output = rewrite_str(input, settings).unwrap();
    assert_eq!(output, input);
    seen.borrow().clone()
}

/// Number of comments the rewriter reports inside elements matching `selector`.
fn comments_in(input: &str, selector: &str) -> usize {
    let count = Rc::new(RefCell::new(0));
    let sink = Rc::clone(&count);
    let settings = RewriteStrSettings::new().append_element_content_handler(comments!(
        selector,
        move |_| {
            *sink.borrow_mut() += 1;
            Ok(())
        }
    ));
    let output = rewrite_str(input, settings).unwrap();
    assert_eq!(output, input);
    count.take()
}

#[test]
fn text_type_follows_the_element_namespace() {
    // The tokenizer's text-type switch is chosen by the insertion mode plus the
    // tag name (13.2.6.4.4 "in head"), never by the enclosing namespace. A tag
    // that reaches the HTML rules switches the tokenizer: `<script>` to script
    // data, `<style>` (and iframe/xmp/noembed/noframes/noscript) to RAWTEXT,
    // `<title>`/`<textarea>` to RCDATA. The same tag inside foreign content is
    // an ordinary foreign element whose content is markup, and the tokenizer
    // stays in the data state. The observable is whether the `<a>` in the text
    // becomes an element at all, and in which namespace.
    let cases = [
        // HTML content: raw text, no element.
        (r#"<script>var s = "<a href='x'>";</script>"#, &[][..]),
        (r#"<style>a[title="<a href='x'>"]{}</style>"#, &[][..]),
        (r#"<title><a href='x'></title>"#, &[][..]),
        (r#"<textarea><a href='x'></textarea>"#, &[][..]),
        // Foreign content: markup, the `<a>` is a foreign element.
        (
            r#"<svg><script>var s = "<a href='x'>";</script></svg>"#,
            &[("a", SVG)][..],
        ),
        (
            r#"<svg><style>a[title="<a href='x'>"]{}</style></svg>"#,
            &[("a", SVG)][..],
        ),
        (
            r#"<math><script>var s = "<a href='x'>";</script></math>"#,
            &[("a", MATHML)][..],
        ),
        // An SVG `<title>` is an integration point, so its content is HTML.
        (
            r#"<svg><title><a href='x'>y</a></title></svg>"#,
            &[("a", HTML)][..],
        ),
        // Children of an integration point follow the HTML rules again.
        (r#"<svg><desc><script>var s = "<a href='x'>";</script></desc></svg>"#, &[][..]),
        (r#"<svg><desc><title><a href='x'></title></desc></svg>"#, &[][..]),
        (r#"<math><mi><script>var s = "<a href='x'>";</script></mi></math>"#, &[][..]),
        (r#"<math><mtext><style>a[title="<a href='x'>"]{}</style></mtext></math>"#, &[][..]),
        // An HTML `<title>` is RCDATA, so a `<script>` inside it is text.
        (
            r#"<svg><foreignObject><title><script>var s = "<a href='x'>";</script></title></foreignObject></svg>"#,
            &[][..],
        ),
        (
            r#"<svg><title><script>var s = "<a href='x'>";</script></title></svg>"#,
            &[][..],
        ),
    ];

    for (input, elements) in cases {
        let (output, seen) = collect(input, "a");
        assert_eq!(seen, expected(elements), "input: {input}");
        assert_eq!(output, input);
    }
}

#[test]
fn mathml_integration_point_end_tag_identity() {
    // `<mi>` inside the MathML text integration point is an HTML element, so
    // its end tag must not close the integration point: the `<script>` after it
    // is still an HTML script.
    let input = r#"<math><mi><mi></mi><script>var s = "<a href='x'>";</script></math>"#;

    let (output, seen) = collect(input, "a");
    assert!(seen.is_empty(), "input: {input}, matched: {seen:?}");
    assert_eq!(output, input);

    let (_, seen) = collect(input, "script");
    assert_eq!(seen, expected(&[("script", HTML)]), "input: {input}");
}

#[test]
fn foreign_content_resumes_after_integration_point_closes() {
    // The mirror case: once the integration point itself is closed, the
    // following `<script>` is foreign content again (markup, not raw text).
    let input = r#"<math><mi></mi><script>var s = "<a href='x'>";</script></math>"#;
    let (output, seen) = collect(input, "a");
    assert_eq!(seen, expected(&[("a", MATHML)]), "input: {input}");
    assert_eq!(output, input);

    // `<img>` is a breakout tag, so it leaves SVG and becomes HTML.
    let input = r#"<svg><title>t</title><script>x<img id=t></script></svg>"#;
    let (output, seen) = collect(input, "img");
    assert_eq!(seen, expected(&[("img", HTML)]), "input: {input}");
    assert_eq!(output, input);
}

#[test]
fn annotation_xml_encoding_selects_the_text_type() {
    // Only `annotation-xml` with an HTML encoding is an integration point.
    let with_encoding = r#"<math><annotation-xml encoding="text/html"><script>var s = "<a href='x'>";</script></annotation-xml></math>"#;
    let (output, seen) = collect(with_encoding, "a");
    assert!(seen.is_empty(), "matched: {seen:?}");
    assert_eq!(output, with_encoding);

    let without_encoding = r#"<math><annotation-xml><script>var s = "<a href='x'>";</script></annotation-xml></math>"#;
    let (_, seen) = collect(without_encoding, "a");
    assert_eq!(seen, expected(&[("a", MATHML)]), "input: {without_encoding}");
}

#[test]
fn comment_syntax_depends_on_the_element_namespace() {
    // `<!--` opens a comment in foreign content but script data in an HTML
    // script, so the same bytes yield an element in one case and none in the
    // other.
    let foreign = r#"<svg><script><!--</script><img src=x onerror=alert(1)></script></svg>"#;
    assert_eq!(comments_in(foreign, "*"), 1, "input: {foreign}");
    let (_, seen) = collect(foreign, "img");
    assert!(seen.is_empty(), "matched: {seen:?}");

    let html = r#"<script><!--</script><img src=x onerror=alert(1)></script>"#;
    assert_eq!(comments_in(html, "*"), 0, "input: {html}");
    let (_, seen) = collect(html, "img");
    assert_eq!(seen, expected(&[("img", HTML)]), "input: {html}");
}

#[test]
fn svg_inside_mathml_is_a_mathml_element() {
    // 13.2.6.5 "Any other start tag": insert a foreign element with the
    // adjusted current node's namespace — MathML here, not SVG.
    let input = r#"<math><svg><g/></svg></math>"#;

    let (output, seen) = collect(input, "*");
    assert_eq!(
        seen,
        expected(&[("math", MATHML), ("svg", MATHML), ("g", MATHML)]),
        "input: {input}"
    );
    assert_eq!(output, input);
}

#[test]
fn foreign_self_closing_script_does_not_change_the_namespace_context() {
    // NOTE: only namespace/text-mode tracking is observable here — the
    // simulator has no element stack, so sibling structure and the effect of
    // the slash on the DOM are out of reach. What is asserted: a self-closing
    // foreign `<script/>` pushes no mode frame, so the following `<a/>` is
    // still foreign content rather than HTML content.
    let input = r#"<svg><script/><a/></svg>"#;

    let (output, seen) = collect(input, "a");
    assert_eq!(seen, expected(&[("a", SVG)]), "input: {input}");
    assert_eq!(output, input);
}

#[test]
fn html_self_closing_script_is_script_data() {
    // Inside an integration point the HTML rules apply: `<script/>` is an HTML
    // script, so the tokenizer enters the script data state and the `<a/>`
    // never becomes an element. (Again a tokenizer-mode assertion, not a DOM-structure one.)
    let input = r#"<svg><desc><script/><a/></desc></svg>"#;

    let (output, seen) = collect(input, "a");
    assert!(seen.is_empty(), "matched: {seen:?}");
    assert_eq!(output, input);

    let (_, seen) = collect(input, "script");
    assert_eq!(seen, expected(&[("script", HTML)]), "input: {input}");
}

#[test]
fn cdata_in_foreign_content_is_text() {
    let input = r#"<svg><![CDATA[x<y]]></svg>"#;

    assert_eq!(comments_in(input, "*"), 0, "input: {input}");
    assert_eq!(text_in(input), "x<y", "input: {input}");
}

#[test]
fn cdata_inside_integration_point_is_text() {
    // The adjusted current node is the SVG `<desc>`, so the tokenizer enters
    // the CDATA section state even though its children are parsed as HTML.
    let input = r#"<svg><desc><![CDATA[x<y]]></desc></svg>"#;

    assert_eq!(comments_in(input, "*"), 0, "input: {input}");
    assert_eq!(text_in(input), "x<y", "input: {input}");
}

#[test]
fn cdata_inside_html_element_in_integration_point_is_a_bogus_comment() {
    // The adjusted current node is now an HTML element, so `<![CDATA[` is a
    // bogus comment, not a CDATA section.
    let input = r#"<svg><desc><p><![CDATA[x<y]]></p></desc></svg>"#;

    assert_eq!(comments_in(input, "*"), 1, "input: {input}");
    assert_eq!(text_in(input), "", "input: {input}");
}

#[test]
fn svg_integration_point_elements_keep_the_svg_namespace() {
    // `desc`/`title`/`foreignObject` are SVG elements; only their children are
    // parsed as HTML.
    let input = r#"<svg><desc></desc><title></title><foreignObject></foreignObject></svg>"#;

    let (output, seen) = collect(input, "*");
    assert_eq!(
        seen,
        expected(&[
            ("svg", SVG),
            ("desc", SVG),
            ("title", SVG),
            ("foreignobject", SVG),
        ]),
        "input: {input}"
    );
    assert_eq!(output, input);
}

#[test]
fn mathml_text_integration_point_elements_keep_the_mathml_namespace() {
    let input = r#"<math><mi>x</mi><mo>+</mo><mn>1</mn><mtext>t</mtext></math>"#;

    let (output, seen) = collect(input, "*");
    assert_eq!(
        seen,
        expected(&[
            ("math", MATHML),
            ("mi", MATHML),
            ("mo", MATHML),
            ("mn", MATHML),
            ("mtext", MATHML),
        ]),
        "input: {input}"
    );
    assert_eq!(output, input);
}

#[test]
fn html_child_of_integration_point_is_an_xhtml_element() {
    // The integration point's own namespace is covered separately; here only
    // the child's namespace matters.
    let input = r#"<svg><desc><p>x</p></desc></svg>"#;

    let (output, seen) = collect(input, "p");
    assert_eq!(seen, expected(&[("p", HTML)]), "input: {input}");
    assert_eq!(output, input);
}

#[test]
fn annotation_xml_without_an_encoding_is_not_an_integration_point() {
    // The `<p>` is a breakout start tag: it pops out of MathML entirely.
    let input = r#"<math><annotation-xml><p>x</p></annotation-xml></math>"#;

    let (output, seen) = collect(input, "*");
    assert_eq!(
        seen,
        expected(&[("math", MATHML), ("annotation-xml", MATHML), ("p", HTML)]),
        "input: {input}"
    );
    assert_eq!(output, input);
}
