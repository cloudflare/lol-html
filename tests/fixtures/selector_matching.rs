use crate::harness::functional_testing::selectors_tests::{get_test_cases, TestCase};
use crate::harness::functional_testing::FunctionalTestFixture;
use crate::harness::Output;
use cool_thing::{ContentType, ElementContentHandlers, HtmlRewriterBuilder};
use encoding_rs::UTF_8;

pub struct SelectorMatchingTests;

impl FunctionalTestFixture<TestCase> for SelectorMatchingTests {
    fn test_cases() -> Vec<TestCase> {
        get_test_cases()
    }

    fn run(test: &TestCase) {
        let mut output = Output::new(UTF_8);
        let mut builder = HtmlRewriterBuilder::default();

        builder
            .on(
                &test.selector,
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
                    }),
            )
            .unwrap();

        {
            let mut rewriter = builder.build("utf-8", |c: &[u8]| output.push(c)).unwrap();

            rewriter.write(test.src.as_bytes()).unwrap();
            rewriter.end().unwrap();
        }

        let actual: String = output.into();

        assert_eq!(actual, test.expected);
    }
}

functional_test_fixture!(SelectorMatchingTests);
