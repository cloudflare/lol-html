use super::Unescape;
use rand::{thread_rng, Rng};
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_json::error::Error;
use std::env;
use std::fmt::{self, Formatter};

#[derive(Debug)]
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
    pub fn get_chunks(&self) -> Vec<&[u8]> {
        vec![self.input.as_bytes()]
        //self.input.as_bytes().chunks(self.chunk_size).collect()
    }

    pub fn get_chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn set_chunk_size(&mut self) {
        let len = self.input.len();

        self.chunk_size = match env::var("CHUNK_SIZE") {
            Ok(val) => val.parse().unwrap(),
            Err(_) => if len > 1 {
                thread_rng().gen_range(1, len)
            } else {
                len
            },
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

            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
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
    fn unescape(&mut self) -> Result<(), Error> {
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
