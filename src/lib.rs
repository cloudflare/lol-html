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

mod content;
mod parser;
mod rewriter;
mod transform_stream;

use cfg_if::cfg_if;

pub use self::rewriter::{
    DocumentContentHandlers, ElementContentHandlers, HtmlRewriter, HtmlRewriterBuilder,
};

pub use self::content::{
    Attribute, AttributeNameError, Comment, CommentTextError, Doctype, Element, TagNameError,
    TextChunk,
};

pub use self::parser::TextType;

cfg_if! {
    if #[cfg(feature = "test_api")] {
        pub use self::transform_stream::{
            ContentSettingsOnElementEnd, ContentSettingsOnElementStart,
            DocumentLevelContentSettings, ElementStartResponse, TransformController,
            TransformStream,
        };

        pub use self::parser::{TagName, TagNameInfo};
        pub use self::content::{EndTag, Serialize, StartTag, Token, TokenCaptureFlags, create_element};
        pub use self::base::Bytes;
    }
}

#[inline]
pub fn html_rewriter<'h>() -> HtmlRewriterBuilder<'h> {
    HtmlRewriterBuilder::new()
}
