//! Namespace tracking in the tree builder simulator.
//!
//! Once the HTML tree-construction algorithm has returned to HTML content, the
//! simulator must agree with it: a following `<script>`/`<style>`/`<title>`
//! belongs to the HTML namespace, so the tokenizer must switch to script
//! data / RAWTEXT / RCDATA for its content. Otherwise markup that only exists
//! as text inside that element is reported as real elements, and rewriting it
//! corrupts the script or stylesheet (or hides a real element from a filter).

use std::cell::RefCell;
use std::rc::Rc;

use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str};

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

#[test]
fn self_closing_foreign_root_returns_to_html_content() {
    // 13.2.6.4.7: `<svg/>`/`<math/>` pops the element immediately, so the
    // following `<script>` is an HTML element.
    let cases = [
        (
            r#"<svg/><script>var s = "<a href='x'>";</script>"#,
            &[("svg", SVG), ("script", HTML)][..],
        ),
        (
            r#"<math/><script>var s = "<a href='x'>";</script>"#,
            &[("math", MATHML), ("script", HTML)][..],
        ),
        (
            r#"<svg><svg/></svg><script>var s = "<a href='x'>";</script>"#,
            &[("svg", SVG), ("svg", SVG), ("script", HTML)][..],
        ),
    ];

    for (input, elements) in cases {
        let (output, seen) = collect(input, "*");
        assert_eq!(seen, expected(elements), "input: {input}");
        assert_eq!(output, input);
    }
}

#[test]
fn text_content_element_after_self_closing_foreign_root_is_not_markup() {
    // `<a>`/`<title>` inside these elements is text, not an element: script
    // data for `<script>`, RAWTEXT for `<style>`, RCDATA for `<title>` and
    // `<textarea>`.
    let cases = [
        (
            r#"<svg/><script>var s = "<a href='x'>";</script>"#,
            "a",
        ),
        (
            r#"<svg/><script>var s = "<title>x</title>";</script>"#,
            "title",
        ),
        (
            r#"<svg/><style>a[title="<a href='x'>"]{}</style>"#,
            "a",
        ),
        (
            r#"<svg/><textarea><!--<a href='x'>--></textarea>"#,
            "a",
        ),
    ];

    for (input, selector) in cases {
        let (output, seen) = collect(input, selector);
        assert!(seen.is_empty(), "input: {input}, matched: {seen:?}");
        assert_eq!(output, input);
    }
}

#[test]
fn font_with_breakout_attributes_leaves_foreign_content() {
    // 13.2.6.5: `<font>` breaks out only when it has a color/face/size
    // attribute; otherwise it is an ordinary foreign element.
    let breaking = r#"<svg><font color=red>x</font><script>var s = "<a href='x'>";</script>"#;

    let (output, seen) = collect(breaking, "font");
    assert_eq!(seen, expected(&[("font", HTML)]), "input: {breaking}");
    let (_, seen) = collect(breaking, "script");
    assert_eq!(seen, expected(&[("script", HTML)]), "input: {breaking}");
    let (_, seen) = collect(breaking, "a");
    assert!(seen.is_empty(), "matched: {seen:?}");
    assert_eq!(output, breaking);

    let plain = r#"<svg><font>x</font><script>var s = "<a href='x'>";</script>"#;

    let (_, seen) = collect(plain, "font");
    assert_eq!(seen, expected(&[("font", SVG)]), "input: {plain}");
    let (_, seen) = collect(plain, "a");
    assert_eq!(seen, expected(&[("a", SVG)]), "input: {plain}");
}

#[test]
fn end_tag_of_enclosing_foreign_element_pops_nested_ones() {
    // 13.2.6.5 "any other end tag": `</math>` pops the nested `<svg>` and then
    // the `<math>` itself, so the following `<script>` is HTML.
    let input = r#"<math><svg></math><script>var s = "<a href='x'>";</script>"#;

    let (output, seen) = collect(input, "script");
    assert_eq!(seen, expected(&[("script", HTML)]), "input: {input}");

    let (_, seen) = collect(input, "a");
    assert!(seen.is_empty(), "matched: {seen:?}");
    assert_eq!(output, input);
}

#[test]
fn foreign_content_breakout_pops_all_foreign_ancestors() {
    // 13.2.6.5: a breakout start tag (here `<div>`) and `</p>` pop every
    // foreign element up to the nearest HTML element or integration point.
    let cases = [
        r#"<svg><svg><div><script>var s = "<a href='x'>";</script>"#,
        r#"<svg><svg></p><script>var s = "<a href='x'>";</script>"#,
    ];

    for input in cases {
        let (_, seen) = collect(input, "script");
        assert_eq!(seen, expected(&[("script", HTML)]), "input: {input}");
    }
}

#[test]
fn end_tag_of_nested_element_named_like_integration_point_keeps_it_open() {
    // `<desc>` is an HTML integration point; the `<title>` inside it is an
    // HTML title. Its end tag closes that title only, so `desc` stays open and
    // the following `<script>` is HTML.
    let input = r#"<svg><desc><title>t</title><script>var s = "<a href='x'>";</script></desc></svg>"#;

    let (output, seen) = collect(input, "script");
    assert_eq!(seen, expected(&[("script", HTML)]), "input: {input}");

    let (_, seen) = collect(input, "title");
    assert_eq!(seen, expected(&[("title", HTML)]), "input: {input}");

    let (_, seen) = collect(input, "a");
    assert!(seen.is_empty(), "matched: {seen:?}");
    assert_eq!(output, input);
}

// The self-closing flag is only observable here through the namespace context,
// because the simulator has no element stack. Content-model cases (script,
// style, title, textarea in each context) live in `foreign_content_edges.rs`.

#[test]
fn self_closing_foreign_elements_stay_in_foreign_content() {
    // The self-closing flag only has to pop a namespace frame when the tag
    // pushed one (svg/math roots, integration points). Ordinary foreign
    // elements push nothing, so the namespace context after them is already
    // correct: the following tag is still foreign content. This says nothing
    // about sibling structure — the simulator has no element stack.
    let cases = [
        (r#"<svg><g/><circle/></svg>"#, "circle", SVG),
        (r#"<math><mi/><b>x</b></math>"#, "b", HTML),
    ];

    for (input, selector, ns) in cases {
        let (output, seen) = collect(input, selector);
        assert_eq!(
            seen,
            expected(&[(selector, ns)]),
            "input: {input}"
        );
        assert_eq!(output, input);
    }
}

#[test]
fn end_tag_inside_integration_point_does_not_close_it() {
    // 13.2.6.4.7 "any other end tag" walks the open elements and stops at the
    // first one in the special category, so `</desc>` only closes the
    // integration point when the integration point is the current node.
    //
    // (1) `<p>` is special: `</desc>` is ignored, `desc` stays open.
    let input = r#"<svg><desc><p></desc><script>var s = "<a href='x'>";</script></desc></svg>"#;
    let (output, seen) = collect(input, "a");
    assert!(seen.is_empty(), "input: {input}, matched: {seen:?}");
    assert_eq!(output, input);

    let (_, seen) = collect(input, "script");
    assert_eq!(seen, expected(&[("script", HTML)]), "input: {input}");

    // (3) `<span>` is not special, but there is no matching element above the
    // integration point either, and the walk stops at the special `desc`.
    let input = r#"<svg><desc><span></desc><script>var s = "<a href='x'>";</script></desc></svg>"#;
    let (output, seen) = collect(input, "a");
    assert!(seen.is_empty(), "input: {input}, matched: {seen:?}");
    assert_eq!(output, input);

    let (_, seen) = collect(input, "script");
    assert_eq!(seen, expected(&[("script", HTML)]), "input: {input}");

    // (2) `</div>` has its own rule and pops through special elements
    // (implied end tags), so `desc` becomes the current node and its end tag
    // closes it: the following `<script>` is foreign content again.
    let input =
        r#"<svg><desc><div><p></div></desc><script>var s = "<a href='x'>";</script></desc></svg>"#;
    let (output, seen) = collect(input, "a");
    assert_eq!(seen, expected(&[("a", SVG)]), "input: {input}");
    assert_eq!(output, input);

    let (_, seen) = collect(input, "script");
    assert_eq!(seen, expected(&[("script", SVG)]), "input: {input}");
}

#[test]
fn end_tag_of_sibling_named_like_integration_point_keeps_integration_point_open() {
    // `</desc>` closes a `desc` that never opened an integration point (a
    // sibling inside `foreignObject`, or a nested HTML `desc` inside an SVG
    // `desc`); it must not close the surrounding integration point.
    let cases = [
        r#"<svg><foreignObject><desc></desc><script>x<img id=t></script></foreignObject></svg>"#,
        r#"<svg><desc><desc></desc><script>x<img id=t></script></desc></svg>"#,
    ];

    for input in cases {
        let (output, seen) = collect(input, "img");
        assert!(seen.is_empty(), "input: {input}, matched: {seen:?}");
        assert_eq!(output, input);

        let (_, seen) = collect(input, "script");
        assert_eq!(seen, expected(&[("script", HTML)]), "input: {input}");
    }
}

#[test]
fn phantom_element_inside_script_cannot_break_out_of_script() {
    let input =
        r#"<svg><foreignObject><desc></desc><script>x<img id=t></script></foreignObject></svg>"#;

    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "img#t",
        |el| {
            el.before("</script><img src=x onerror=alert(1)>", ContentType::Html);
            Ok(())
        }
    ));

    assert_eq!(rewrite_str(input, settings).unwrap(), input);
}

#[test]
fn element_handler_sees_img_after_script_with_escaped_comment() {
    // The browser's script data escaped state ends the script at `</script>`,
    // so this `<img>` is a real element with an `onerror` handler.
    let input = r#"<svg/><script><!--</script><img src=x onerror=alert(1)>--></script>"#;

    let (_, seen) = collect(input, "img");
    assert_eq!(seen, expected(&[("img", HTML)]), "input: {input}");
}

#[test]
fn element_handler_sees_img_after_style_with_comment() {
    let input = r#"<svg/><style><!--</style><img src=x onerror=alert(1)>"#;

    let (_, seen) = collect(input, "img");
    assert_eq!(seen, expected(&[("img", HTML)]), "input: {input}");
}
