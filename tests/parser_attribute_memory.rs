use lol_html::errors::RewritingError;
use lol_html::{HtmlRewriter, MemorySettings, Settings, element};
use std::cell::Cell;

#[test]
fn attribute_metadata_obeys_memory_limit() {
    const LIMIT: usize = 64 * 1024;
    for attribute in [" a", " a=''", " a=\"\"", " a=x"] {
        let input = format!(
            "<span>before</span><div{}>after</div>",
            attribute.repeat(4096)
        );
        assert!(input.len() < LIMIT);
        for chunk_size in [1, 7, 512, 4096, input.len()] {
            for graceful in [false, true] {
                let called = Cell::new(false);
                let mut output = Vec::new();
                let mut failed = false;
                let mut fed_bytes = 0;
                {
                    let settings = Settings::new()
                        .with_memory_settings(
                            MemorySettings::new()
                                .with_preallocated_parsing_buffer_size(0)
                                .with_max_allowed_memory_usage(LIMIT)
                                .with_graceful_bail_out_on_memory_limit_exceeded(graceful),
                        )
                        .append_element_content_handler(element!("div", |_| {
                            called.set(true);
                            Ok(())
                        }));
                    let mut rewriter = HtmlRewriter::new(settings, |bytes: &[u8]| {
                        output.extend_from_slice(bytes);
                    });
                    for chunk in input.as_bytes().chunks(chunk_size) {
                        fed_bytes += chunk.len();
                        if let Err(err) = rewriter.write(chunk) {
                            assert!(matches!(err, RewritingError::MemoryLimitExceeded(_)));
                            assert!(!called.get());
                            failed = true;
                            break;
                        }
                    }
                }
                assert!(failed, "{attribute}, chunk size {chunk_size}");
                if graceful {
                    output.extend_from_slice(&input.as_bytes()[fed_bytes..]);
                    assert_eq!(output, input.as_bytes());
                }
            }
        }
    }
}

#[test]
fn unique_attribute_metadata_obeys_memory_limit() {
    let attributes: String = (0..4096).map(|i| format!(" a{i}")).collect();
    let input = format!("<div{attributes}>after</div>");
    assert!(input.len() < 64 * 1024);
    let settings = Settings::new()
        .with_memory_settings(MemorySettings::new().with_max_allowed_memory_usage(64 * 1024))
        .append_element_content_handler(element!("[onload]", |_| Ok(())));
    let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});
    assert!(matches!(
        rewriter.write(input.as_bytes()),
        Err(RewritingError::MemoryLimitExceeded(_))
    ));
}

#[test]
fn attribute_memory_is_released_between_tags() {
    let tag = format!("<div{}></div>", " a".repeat(64));
    let input = tag.repeat(1000);
    for chunk_size in [1, 512, input.len()] {
        let mut output = Vec::new();
        let settings = Settings::new()
            .with_memory_settings(
                MemorySettings::new()
                    .with_preallocated_parsing_buffer_size(0)
                    .with_max_allowed_memory_usage(16 * 1024),
            )
            .append_element_content_handler(element!("div", |_| Ok(())));
        {
            let mut rewriter =
                HtmlRewriter::new(settings, |bytes: &[u8]| output.extend_from_slice(bytes));
            for chunk in input.as_bytes().chunks(chunk_size) {
                rewriter.write(chunk).unwrap();
            }
            rewriter.end().unwrap();
        }
        assert_eq!(output, input.as_bytes());
    }
}

#[test]
fn raw_token_and_attributes_share_the_memory_limit() {
    let input = format!(
        "<div{} z='{}'>body</div>",
        " a".repeat(128),
        "x".repeat(12000)
    );
    for chunk_size in [512, input.len()] {
        let settings = Settings::new()
            .with_memory_settings(MemorySettings::new().with_max_allowed_memory_usage(16 * 1024))
            .append_element_content_handler(element!("div", |_| Ok(())));
        let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});
        let result = input
            .as_bytes()
            .chunks(chunk_size)
            .try_for_each(|chunk| rewriter.write(chunk));
        if chunk_size == input.len() {
            result.unwrap();
            rewriter.end().unwrap();
        } else {
            assert!(matches!(
                result,
                Err(RewritingError::MemoryLimitExceeded(_))
            ));
        }
    }
}
