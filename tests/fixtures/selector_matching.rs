use crate::harness::suites::selectors_tests::{get_test_cases, TestCase};
use crate::harness::TestFixture;
use cool_thing::test_utils::Output;
use cool_thing::{ContentType, ElementContentHandlers, HtmlRewriter, Settings};
use std::convert::TryFrom;

pub struct SelectorMatchingTests;

impl TestFixture<TestCase> for SelectorMatchingTests {
    fn test_cases() -> Vec<TestCase> {
        get_test_cases("selector_matching")
    }

    fn run(test: &TestCase) {
        let encoding = test.input.encoding().unwrap();
        let mut output = Output::new(encoding);
        let mut first_text_chunk_expected = true;

        {
            let mut rewriter = HtmlRewriter::try_from(Settings {
                element_content_handlers: vec![(
                    &test.selector.parse().unwrap(),
                    ElementContentHandlers::default()
                        .element(|el| {
                            el.before(
                                &format!("<!--[ELEMENT('{}')]-->", test.selector),
                                ContentType::Html,
                            );
                            el.after(
                                &format!("<!--[/ELEMENT('{}')]-->", test.selector),
                                ContentType::Html,
                            );

                            Ok(())
                        })
                        .comments(|c| {
                            c.before(
                                &format!("<!--[COMMENT('{}')]-->", test.selector),
                                ContentType::Html,
                            );
                            c.after(
                                &format!("<!--[/COMMENT('{}')]-->", test.selector),
                                ContentType::Html,
                            );

                            Ok(())
                        })
                        .text(|t| {
                            if first_text_chunk_expected {
                                t.before(
                                    &format!("<!--[TEXT('{}')]-->", test.selector),
                                    ContentType::Html,
                                );

                                first_text_chunk_expected = false;
                            }

                            if t.last_in_text_node() {
                                t.after(
                                    &format!("<!--[/TEXT('{}')]-->", test.selector),
                                    ContentType::Html,
                                );

                                first_text_chunk_expected = true;
                            }

                            Ok(())
                        }),
                )],
                document_content_handlers: vec![],
                encoding: encoding.name(),
                max_memory: 200 * 1024,
                preallocated_memory: 0,
                output_sink: |c: &[u8]| output.push(c),
                strict: true,
            })
            .unwrap();

            for chunk in test.input.chunks() {
                rewriter.write(chunk).unwrap();
            }

            rewriter.end().unwrap();
        }

        let actual: String = output.into();

        assert_eq!(actual, test.expected);
    }
}

test_fixture!(SelectorMatchingTests);
