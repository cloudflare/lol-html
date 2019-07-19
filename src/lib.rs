#[macro_use]
extern crate failure;

#[macro_use]
mod base;

#[macro_use]
mod html;

mod parser;
mod rewritable_units;
mod rewriter;
mod transform_stream;

use cfg_if::cfg_if;

pub use self::rewriter::{
    DocumentContentHandlers, ElementContentHandlers, EncodingError, HtmlRewriter, Settings,
};

pub use self::rewritable_units::{
    Attribute, AttributeNameError, Comment, CommentTextError, ContentType, Doctype, Element,
    TagNameError, TextChunk,
};

pub use self::html::TextType;
pub use self::selectors_vm::{Selector, SelectorError};
pub use self::transform_stream::OutputSink;

cfg_if! {
    if #[cfg(feature = "test_api")] {
        pub mod selectors_vm;

        pub use self::transform_stream::{
            AuxStartTagInfo, StartTagHandlingResult, TransformController, TransformStream,
        };

        pub use self::rewritable_units::{
            EndTag, Serialize, StartTag, Token, TokenCaptureFlags, Mutations
        };

        pub use self::base::{Bytes, BufferCapacityExceededError};
        pub use self::html::{LocalName, LocalNameHash, Tag, Namespace, TAG_STR_PAIRS};

    } else {
        mod selectors_vm;
    }
}
