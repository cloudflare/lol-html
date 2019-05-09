use super::{for_each_test_file, get_test_file_reader};
use cool_thing::selectors_vm::SelectorsParser;
use serde::de::{Deserialize, Deserializer};
use serde_json::{self, from_reader};
use std::collections::HashMap;
use std::io::prelude::*;

fn read_test_file(name: &str) -> String {
    let mut data = String::new();

    get_test_file_reader(&format!("selectors/{}", name))
        .read_to_string(&mut data)
        .unwrap();

    data
}

fn read_src<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;

    Ok(read_test_file(&name))
}

#[derive(Deserialize)]
struct TestData {
    pub description: String,
    pub selectors: HashMap<String, String>,

    #[serde(deserialize_with = "read_src")]
    pub src: String,
}

pub struct TestCase {
    pub description: String,
    pub selector: String,
    pub src: String,
    pub expected: String,
}

pub fn get_test_cases() -> Vec<TestCase> {
    let mut test_cases = Vec::new();
    let mut ignored_count = 0;

    for_each_test_file("selectors/*-info.json", &mut |file| {
        let test_data = from_reader::<_, TestData>(file).unwrap();

        for (selector, expected_file) in test_data.selectors {
            let description = format!("{} (`{}`)", test_data.description, selector);

            if SelectorsParser::parse(&selector).is_err() {
                ignore!(@info
                    "Ignoring test due to unsupported selector: `{}`",
                    description
                );

                ignored_count += 1;

                continue;
            }

            test_cases.push(TestCase {
                description,
                selector,
                src: test_data.src.to_owned(),
                expected: read_test_file(&expected_file),
            });
        }
    });

    ignore!(@total "selector matching", ignored_count);

    test_cases
}
