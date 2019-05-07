#[macro_use]
mod mutations;

mod element;
mod tokens;

use crate::base::Bytes;
use encoding_rs::Encoding;

pub use self::element::*;
pub use self::mutations::Mutations;
pub use self::tokens::*;

pub enum ContentType {
    Html,
    Text,
}

#[inline]
fn content_to_bytes(
    content: &str,
    content_type: ContentType,
    encoding: &'static Encoding,
    output_handler: &mut dyn FnMut(&[u8]),
) {
    let bytes = Bytes::from_str(content, encoding);

    match content_type {
        ContentType::Html => output_handler(&bytes),
        ContentType::Text => bytes.replace_byte3(
            (b'<', b"&lt;"),
            (b'>', b"&gt;"),
            (b'&', b"&amp;"),
            output_handler,
        ),
    }
}
