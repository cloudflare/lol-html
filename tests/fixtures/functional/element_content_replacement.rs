
use crate::harness::functional_testing::selectors_tests::{get_test_cases, TestCase};
use crate::harness::functional_testing::FunctionalTestFixture;
use crate::harness::Output;
use cool_thing::{ContentType, ElementContentHandlers, HtmlRewriter, Settings};
use std::convert::TryFrom;

// NOTE: Inner element content replacement functionality used as a basis for
// the multiple element methods and it's easy to get it wrong, so we have
// a dedicated set of functional tests for that.
pub struct ElementContentReplacementTests;

impl FunctionalTestFixture<TestCase> for ElementContentReplacementTests {
    fn test_cases() -> Vec<TestCase> {
        get_test_cases("element_content_replacement")
    }

    fn run(test: &TestCase) {
        let encoding = test.input.encoding().unwrap();
        let mut output = Output::new(encoding);

        {
            let mut rewriter = HtmlRewriter::try_from(Settings{
                element_content_handlers: vec![(
                    &test.selector.parse().unwrap(),
                    ElementContentHandlers::default().element(|el| {
                        el.set_inner_content(
                            &format!("<!--Replaced ({}) -->", test.selector),
                            ContentType::Html,
                        );

                        Ok(())
                    }),
                )],
                document_content_handlers: vec![],
                encoding: encoding.name(),
                buffer_capacity: 2048,
                output_sink: |c: &[u8]| output.push(c),
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

functional_test_fixture!(ElementContentReplacementTests);
