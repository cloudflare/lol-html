#[macro_use]
mod common;

pub mod eager;
pub mod full;

pub use self::eager::{EagerStateMachine, TagPreviewSink};
pub use self::full::{FullStateMachine, LexUnitSink};
