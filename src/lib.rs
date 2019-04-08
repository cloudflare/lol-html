// TODO test all errors!!!x

// TODO
// -- Functionality
// 5. Adjustable limits
//
// -- Performance
// 5. Don't emit character immidiately, extend existing
// 6. State embedding
// 7. Grow the buffer lazily

// 7. We can use fast skip if:
// there is _ => () branch
// there are no consequent range or sequence branches
// If there is only one character branch except _, eof or eoc the use memchr
// Otherwise find the biggest char in the seq of skippable chars, use bit vector
// for skippable chars and compare that it less than 64.
// Try single loop

// 8.Lazily initialize buffer
// 9.Use smaller buffer for attributes (default?), it will grow proportional to
// to the buffer size, add the comment.
#[macro_use]
extern crate failure;

#[macro_use]
mod debug_trace;

#[macro_use]
mod base;

#[macro_use]
mod html;

mod parser;
mod rewritable_units;
mod rewriter;
mod transform_stream;
mod virtual_tree;

use cfg_if::cfg_if;

pub use self::rewriter::{
    DocumentContentHandlers, ElementContentHandlers, EncodingError, HtmlRewriter,
    HtmlRewriterBuilder, SelectorError,
};

pub use self::rewritable_units::{
    Attribute, AttributeNameError, Comment, CommentTextError, ContentType, Doctype, Element,
    TagNameError, TextChunk,
};

pub use self::html::TextType;
pub use self::transform_stream::OutputSink;

cfg_if! {
    if #[cfg(feature = "test_api")] {
        pub use self::transform_stream::{
            ElementStartHandlingResult, TransformController, TransformStream,
        };

        pub use self::rewritable_units::{
            EndTag, Serialize, StartTag, Token, TokenCaptureFlags, create_element,
        };

        pub use self::base::Bytes;
        pub use self::html::{LocalName, LocalNameHash, Tag, Namespace, TAG_STR_PAIRS};
    }
}
