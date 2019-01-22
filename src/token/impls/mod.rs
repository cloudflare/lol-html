mod comment;
mod doctype;
mod tags;
mod text;

pub use self::comment::{Comment, CommentTextError};
pub use self::doctype::Doctype;
pub use self::tags::*;
pub use self::text::Text;
