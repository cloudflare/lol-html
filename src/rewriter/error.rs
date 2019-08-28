use failure::Error;

#[derive(Fail, Debug)]
pub enum RewriterError {
    #[fail(display = "{}", _0)]
    EncodingError(EncodingError),
    #[fail(display = "{}", _0)]
    InvalidSettings(String),
    #[fail(display = "{}", _0)]
    Fatal(Error),
}

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum EncodingError {
    #[fail(display = "Unknown character encoding has been provided.")]
    UnknownEncoding,
    #[fail(display = "Expected ASCII-compatible encoding.")]
    NonAsciiCompatibleEncoding,
}

impl From<EncodingError> for RewriterError {
    fn from(err: EncodingError) -> Self {
        RewriterError::EncodingError(err)
    }
}

impl From<Error> for RewriterError {
    fn from(err: Error) -> Self {
        RewriterError::Fatal(err)
    }
}
