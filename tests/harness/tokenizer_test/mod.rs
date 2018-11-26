mod chunked_input;
mod decoder;
mod runners;
mod test_outputs;
mod unescape;

use self::chunked_input::ChunkedInput;
use self::unescape::Unescape;
use serde_json;
use std::fmt::Write;

pub use self::runners::{
    EagerStateMachineTestRunner, FullStateMachineTestRunner, StateMachineSwitchTestRunner,
    TokenizerTestRunner,
};
pub use self::test_outputs::TestToken;

pub fn default_initial_states() -> Vec<String> {
    vec![String::from("Data state")]
}

#[derive(Deserialize, Default, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bailout {
    pub reason: String,
    pub parsed_chunk: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenizerTest {
    pub description: String,
    pub input: ChunkedInput,

    #[serde(rename = "output")]
    pub expected_tokens: Vec<TestToken>,

    #[serde(default = "default_initial_states")]
    pub initial_states: Vec<String>,

    #[serde(default)]
    pub double_escaped: bool,

    #[serde(default)]
    pub last_start_tag: String,

    #[serde(default)]
    pub expected_tag_tokens: Vec<TestToken>,

    #[serde(skip)]
    pub ignored: bool,

    #[serde(skip)]
    pub expected_bailout: Option<Bailout>,
}

impl Unescape for TokenizerTest {
    fn unescape(&mut self) -> Result<(), serde_json::error::Error> {
        if self.double_escaped {
            self.double_escaped = false;
            self.input.unescape()?;

            for token in &mut self.expected_tokens {
                token.unescape()?;
            }
        }

        Ok(())
    }
}

impl TokenizerTest {
    pub fn init(&mut self) {
        self.ignored = self.unescape().is_err();

        // NOTE: tokenizer should always produce EOF token
        self.expected_tokens.push(TestToken::Eof);

        let mut new_descr = String::new();

        write!(
            &mut new_descr,
            "`{}` (chunk size: {})",
            self.description,
            self.input.get_chunk_size()
        ).unwrap();

        self.description = new_descr;

        self.expected_tag_tokens = self
            .expected_tokens
            .to_owned()
            .into_iter()
            .filter(|t| match t {
                TestToken::StartTag { .. } | TestToken::EndTag { .. } => true,
                _ => false,
            }).collect::<Vec<_>>();
    }
}

macro_rules! add_test {
    ($tests:ident, $t:ident, $runner:ident) => {{
        let t = $t.clone();

        $tests.push(create_test!(
            $runner::get_test_description(&t),
            t.ignored,
            {
                $runner::run(&t);
            }
        ))
    }};
}

macro_rules! tokenizer_tests {
    ($tests:expr) => {{
        use harness::tokenizer_test::{
            EagerStateMachineTestRunner, FullStateMachineTestRunner, StateMachineSwitchTestRunner,
            TokenizerTestRunner,
        };

        $tests.into_iter().fold(Vec::new(), |mut tests, mut t| {
            t.init();

            if t.ignored {
                println!("Ignoring test: `{}`", t.description);
            }

            add_test!(tests, t, EagerStateMachineTestRunner);
            add_test!(tests, t, FullStateMachineTestRunner);
            add_test!(tests, t, StateMachineSwitchTestRunner);

            tests
        })
    }};
}
