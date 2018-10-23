#[derive(Debug, Copy, Clone)]
pub enum TransformBailoutReason {
    BufferCapacityExceeded,
    TextParsingAmbiguity,
    MaxTagNestingReached,
}
