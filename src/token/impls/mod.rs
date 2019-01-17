mod comment;
mod doctype;
mod end_tag;
mod start_tag;
mod text;

pub use self::comment::Comment;
pub use self::doctype::Doctype;
pub use self::end_tag::EndTag;
pub use self::start_tag::*;
pub use self::text::Text;
