use super::{for_each_test_file, get_test_file_reader};
use serde::de::{Deserialize, Deserializer};
use serde_json::{self, from_reader};
use std::io::prelude::*;

fn read_data_file<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    let mut data = String::new();

    get_test_file_reader(&format!("selectors/{}", name))
        .read_to_string(&mut data)
        .unwrap();

    Ok(data)
}

#[derive(Deserialize)]
pub struct TestCase {
    pub description: String,
    pub selectors: Vec<String>,

    #[serde(deserialize_with = "read_data_file")]
    pub src: String,

    #[serde(deserialize_with = "read_data_file")]
    pub expected: String,
}

pub fn get_test_cases() -> Vec<TestCase> {
    let mut test_cases = Vec::new();

    for_each_test_file("selectors/*-info.json", &mut |file| {
        test_cases.push(from_reader::<_, TestCase>(file).unwrap());
    });

    test_cases
}
