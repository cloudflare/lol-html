use super::*;
use crate::base::Bytes;
use encoding_rs::Encoding;
use failure::Error;

// TODO validations
pub struct TokenFactory {
    encoding: &'static Encoding,
}

impl TokenFactory {
    pub fn new(encoding: &'static Encoding) -> Self {
        TokenFactory { encoding }
    }

    pub fn new_attribute(&self, name: &str, value: &str) -> Result<Attribute<'static>, Error> {
        Ok(Attribute::new(
            Attribute::name_from_str(name, self.encoding)?,
            Bytes::from_str(value, self.encoding).into_owned(),
            None,
            self.encoding,
        ))
    }
}
