use super::{for_each_test_file, get_test_file_reader};
use crate::harness::Input;
use hashbrown::HashMap;
use lol_html::test_utils::ASCII_COMPATIBLE_ENCODINGS;
use lol_html::Selector;
use serde_derive::Deserialize;
use serde_json::{self, from_reader};
use std::io::prelude::*;

fn read_test_file(suite: &'static str, name: &str) -> String {
    let mut data = String::new();

    get_test_file_reader(&format!("{suite}/{name}"))
        .read_to_string(&mut data)
        .unwrap();

    data
}

#[derive(Deserialize)]
struct TestData {
    pub description: String,
    pub selectors: HashMap<String, String>,
    pub src: String,
}

#[derive(Debug)]
pub struct TestCase {
    pub _description: String,
    pub selector: String,
    pub input: Input,
    pub expected: String,
}

pub fn get_test_cases(suite: &'static str) -> Vec<TestCase> {
    let mut test_cases = Vec::new();
    let mut ignored_count = 0;

    for_each_test_file(&format!("{suite}/*-info.json"), &mut |file| {
        let test_data = from_reader::<_, TestData>(file).unwrap();
        let src_data = read_test_file(suite, &test_data.src);
        let input = Input::from(src_data);

        for (selector, expected_file) in test_data.selectors {
            for encoding in &ASCII_COMPATIBLE_ENCODINGS {
                let mut input = input.clone();
                let chunk_size = input.init(encoding, false).unwrap();

                let description = format!(
                    "{} ({}) - Encoding: {} - Chunk size: {}",
                    test_data.description,
                    selector,
                    encoding.name(),
                    chunk_size
                );

                if selector.parse::<Selector>().is_err() {
                    ignore!(@info
                        "Ignoring test due to unsupported selector: `{}`",
                        description
                    );

                    ignored_count += 1;

                    continue;
                }

                test_cases.push(TestCase {
                    _description: description,
                    selector: selector.clone(),
                    input,
                    expected: read_test_file(suite, &expected_file),
                });
            }
        }
    });

    ignore!(@total suite, ignored_count);

    test_cases
}
