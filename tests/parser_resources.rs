use lol_html::{HtmlRewriter, Settings, doc_comments, doctype, element};

fn rewrite(input: &[u8], chunk_size: usize, capture_tokens: bool) -> Vec<u8> {
    let mut output = Vec::new();
    let mut settings = Settings::new()
        .with_strict(false)
        .append_element_content_handler(element!("div", |el| {
            el.set_attribute("data-rewritten", "yes")?;
            Ok(())
        }));
    if capture_tokens {
        settings = settings
            .append_document_content_handler(doc_comments!(|_| Ok(())))
            .append_document_content_handler(doctype!(|_| Ok(())));
    }
    {
        let mut rewriter = HtmlRewriter::new(settings, |bytes: &[u8]| {
            output.extend_from_slice(bytes);
        });
        for chunk in input.chunks(chunk_size) {
            rewriter.write(chunk).unwrap();
        }
        rewriter.end().unwrap();
    }
    output
}

#[test]
fn attributes_survive_zero_and_nonzero_input_shifts() {
    let attributes: String = (0..128).map(|i| format!(" a{i}='v{i}'")).collect();
    let input = format!(
        "<!DOCTYPE html><!--before--><span>prefix</span><div{attributes}>body</div>\
         <textarea>x &amp; y</textarea><script>if (a < b) x='</not-script>';</script>\
         <svg><title>svg</title></svg>"
    );
    for capture_tokens in [false, true] {
        let expected = rewrite(input.as_bytes(), input.len(), capture_tokens);
        let expected_text = std::str::from_utf8(&expected).unwrap();
        assert!(expected_text.contains("a127='v127'") || expected_text.contains("a127=\"v127\""));
        for chunk_size in [1, 2, 3, 7, 64, 512, 4096] {
            assert_eq!(
                rewrite(input.as_bytes(), chunk_size, capture_tokens),
                expected,
                "chunk size {chunk_size}, capture tokens {capture_tokens}"
            );
        }
    }
}

#[test]
fn incomplete_tokens_survive_zero_and_nonzero_input_shifts() {
    for token in [
        "<div a='value' b=\"unfinished",
        "</div unfinished='value",
        "<!--unfinished comment",
        "<!DOCTYPE html PUBLIC 'unfinished",
        "<script>unfinished </scr",
        "<textarea>unfinished &am",
        "<svg><![CDATA[unfinished]]",
    ] {
        for prefix in ["", "<p>prefix</p>"] {
            let input = format!("{prefix}{token}");
            for capture_tokens in [false, true] {
                let expected = rewrite(input.as_bytes(), input.len(), capture_tokens);
                for chunk_size in [1, 2, 3, 7, 64] {
                    assert_eq!(
                        rewrite(input.as_bytes(), chunk_size, capture_tokens),
                        expected,
                        "{input}, chunk size {chunk_size}, capture tokens {capture_tokens}"
                    );
                }
            }
        }
    }
}
