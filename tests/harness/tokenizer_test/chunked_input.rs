use super::Unescape;
use cool_thing::tokenizer::{
    LexUnitHandler, NextOutputType, TagLexUnitHandler, TagPreviewHandler, TextParsingModeSnapshot,
};
use cool_thing::transform_stream::TransformStream;
use cool_thing::Error;
use rand::{thread_rng, Rng};
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_json::error::Error as SerdeError;
use std::env;
use std::fmt::{self, Formatter};

#[derive(Debug, Clone)]
pub struct ChunkedInput {
    input: String,
    chunk_size: usize,
}

impl From<String> for ChunkedInput {
    fn from(input: String) -> Self {
        let mut input = ChunkedInput {
            input,
            chunk_size: 1,
        };

        input.set_chunk_size();

        input
    }
}

impl ChunkedInput {
    pub fn get_chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn parse<LH, TH, PH>(
        &self,
        mut transform_stream: TransformStream<LH, TH, PH>,
        initial_mode_snapshot: TextParsingModeSnapshot,
        initial_output_type: NextOutputType,
    ) -> Result<(), Error>
    where
        LH: LexUnitHandler,
        TH: TagLexUnitHandler,
        PH: TagPreviewHandler,
    {
        {
            let tokenizer = transform_stream.get_tokenizer();

            tokenizer.set_next_output_type(initial_output_type);
            tokenizer.set_last_start_tag_name_hash(initial_mode_snapshot.last_start_tag_name_hash);
            tokenizer.switch_text_parsing_mode(initial_mode_snapshot.mode);
        }

        for chunk in self.get_chunks() {
            transform_stream.write(chunk)?;
        }

        transform_stream.end()?;

        Ok(())
    }

    fn get_chunks(&self) -> Vec<&[u8]> {
        let bytes = self.input.as_bytes();

        if self.chunk_size > 0 {
            bytes.chunks(self.chunk_size).collect()
        } else {
            vec![bytes]
        }
    }

    fn set_chunk_size(&mut self) {
        let len = self.input.len();

        self.chunk_size = match env::var("CHUNK_SIZE") {
            Ok(val) => val.parse().unwrap(),
            Err(_) => {
                if len > 1 {
                    thread_rng().gen_range(1, len)
                } else {
                    len
                }
            }
        };
    }
}

impl<'de> Deserialize<'de> for ChunkedInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> Visitor<'de> for StringVisitor {
            type Value = ChunkedInput;

            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value.to_owned().into())
            }
        }

        deserializer.deserialize_string(StringVisitor)
    }
}

impl Unescape for ChunkedInput {
    fn unescape(&mut self) -> Result<(), SerdeError> {
        self.input.unescape()?;
        self.set_chunk_size();

        Ok(())
    }
}

impl PartialEq<ChunkedInput> for String {
    fn eq(&self, value: &ChunkedInput) -> bool {
        *self == value.input
    }
}
