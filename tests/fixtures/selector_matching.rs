use crate::harness::functional_testing::selectors_tests::{get_test_cases, TestCase};
use crate::harness::functional_testing::FunctionalTestFixture;

pub struct SelectorMatchingTests;

impl FunctionalTestFixture<TestCase> for SelectorMatchingTests {
    fn test_cases() -> Vec<TestCase> {
        get_test_cases()
    }

    fn run(_test: &TestCase) {
        //TODO
    }
}

functional_test_fixture!(SelectorMatchingTests);
