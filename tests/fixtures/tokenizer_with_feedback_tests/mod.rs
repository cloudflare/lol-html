mod feedback_tokens;

use self::feedback_tokens::get_expected_tokens_with_feedback;
use glob;
use harness::tokenizer_test::{default_initial_states, TokenizerTest};
use std::fs::File;
use std::io::{BufRead, BufReader};
use test::TestDescAndFn;

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

pub fn get_tests() -> Vec<TestDescAndFn> {
    let mut tests = Vec::new();

    for test_files in vec![
        read_test_data!("html5lib-tests/tree-construction/*.dat"),
        read_test_data!("regression/*.dat"),
    ] {
        for file in test_files {
            tests.extend(parse_inputs(file).into_iter().map(|input| {
                TokenizerTest {
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
    }

    convert_tokenizer_tests!(tests)
}
