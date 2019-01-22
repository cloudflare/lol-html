mod comment;
mod doctype;
mod tags;
mod text_chunk;

pub use self::comment::{Comment, CommentTextError};
pub use self::doctype::Doctype;
pub use self::tags::*;
pub use self::text_chunk::TextChunk;
