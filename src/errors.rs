#[derive(Debug, Copy, Clone)]
pub enum Error {
    BufferCapacityExceeded,
    TextParsingAmbiguity,
    MaxTagNestingReached,
}
