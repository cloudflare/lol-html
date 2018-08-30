mod feedback_tokens;

use self::feedback_tokens::get_expected_tokens_with_feedback;
use glob;
use harness::test::{default_initial_states, Test};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn parse_inputs(file: BufReader<File>) -> Vec<String> {
    let mut inputs = Vec::new();
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

pub fn get_tests() -> Vec<Test> {
    let mut tests = Vec::new();

    for file in read_tests!("html5lib-tests/tree-construction/*.dat") {
        tests.extend(parse_inputs(file).into_iter().map(|input| {
            Test {
                description: input
                    .chars()
                    .flat_map(|c| c.escape_default())
                    .collect::<String>() + " (with feedback)",
                expected_tokens: get_expected_tokens_with_feedback(&input),
                input,
                initial_states: default_initial_states(),
                double_escaped: false,
                last_start_tag: String::new(),
                ignored: false,
            }
        }));
    }

    tests
}
