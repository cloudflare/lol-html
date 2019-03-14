mod feedback_tokens;

use self::feedback_tokens::get_expected_tokens_with_feedback;
use crate::harness::functional_testing::{default_initial_states, Bailout, TestCase};
use glob;
use serde_json::from_reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn parse_inputs(file: BufReader<File>) -> Vec<String> {
    let mut inputs = Vec::default();
    let mut in_data = 0;

    for line in file.lines().map(|line| line.unwrap()) {
        if line == "#data" {
            in_data = 1;
        } else if line.starts_with('#') {
            in_data = 0;
        } else if in_data > 0 {
            if in_data > 1 {
                let s: &mut String = inputs.last_mut().unwrap();
                s.push('\n');
                s.push_str(&line);
            } else {
                inputs.push(line);
            }
            in_data += 1;
        }
    }

    inputs
}

#[derive(Deserialize, Default)]
pub struct ExpectedBailouts(HashMap<String, Bailout>);

fn load_expected_bailouts() -> ExpectedBailouts {
    let file = read_test_data!("expected_bailouts.json").pop().unwrap();

    from_reader::<_, ExpectedBailouts>(file).unwrap()
}

pub fn get_test_cases() -> Vec<TestCase> {
    let mut tests = Vec::default();
    let expected_bailouts = load_expected_bailouts();

    for test_files in vec![
        read_test_data!("html5lib-tests/tree-construction/*.dat"),
        read_test_data!("regression/*.dat"),
    ] {
        for file in test_files {
            tests.extend(parse_inputs(file).into_iter().map(|input| {
                let expected_bailout = expected_bailouts.0.get(&input).cloned();

                TestCase {
                    description: input
                        .chars()
                        .flat_map(|c| c.escape_default())
                        .collect::<String>()
                        + " (with feedback)",
                    expected_tokens: get_expected_tokens_with_feedback(&input),
                    input: input.into(),
                    initial_states: default_initial_states(),
                    double_escaped: false,
                    last_start_tag: String::new(),
                    expected_bailout,
                }
            }));
        }
    }

    tests
}
