#[macro_use]
extern crate failure;

#[macro_use]
mod base;

#[macro_use]
mod html;

#[macro_use]
mod rewriter;

mod memory;
mod parser;
mod rewritable_units;
mod transform_stream;

use cfg_if::cfg_if;

pub use self::rewriter::{
    rewrite_str, DocumentContentHandlers, ElementContentHandlers, HtmlRewriter, MemorySettings,
    RewriteStrSettings, Settings,
};
pub use self::selectors_vm::Selector;
pub use self::transform_stream::OutputSink;

/// The errors that can be produced by the crate's API.
pub mod errors {
    pub use super::memory::MemoryLimitExceededError;
    pub use super::parser::ParsingAmbiguityError;
    pub use super::rewritable_units::{AttributeNameError, CommentTextError, TagNameError};
    pub use super::rewriter::{EncodingError, RewritingError};
    pub use super::selectors_vm::SelectorError;
}

/// HTML content descriptors that can be produced and modified by a rewriter.
pub mod html_content {
    pub use super::rewritable_units::{
        Attribute, Comment, ContentType, Doctype, Element, TextChunk, UserData,
    };

    pub use super::html::TextType;
}

#[cfg(any(test, feature = "integration_test"))]
pub mod test_utils {
    use encoding_rs::*;

    pub static ASCII_COMPATIBLE_ENCODINGS: [&Encoding; 36] = [
        BIG5,
        EUC_JP,
        EUC_KR,
        GB18030,
        GBK,
        IBM866,
        ISO_8859_2,
        ISO_8859_3,
        ISO_8859_4,
        ISO_8859_5,
        ISO_8859_6,
        ISO_8859_7,
        ISO_8859_8,
        ISO_8859_8_I,
        ISO_8859_10,
        ISO_8859_13,
        ISO_8859_14,
        ISO_8859_15,
        ISO_8859_16,
        KOI8_R,
        KOI8_U,
        MACINTOSH,
        SHIFT_JIS,
        UTF_8,
        WINDOWS_874,
        WINDOWS_1250,
        WINDOWS_1251,
        WINDOWS_1252,
        WINDOWS_1253,
        WINDOWS_1254,
        WINDOWS_1255,
        WINDOWS_1256,
        WINDOWS_1257,
        WINDOWS_1258,
        X_MAC_CYRILLIC,
        X_USER_DEFINED,
    ];

    pub struct Output {
        bytes: Vec<u8>,
        encoding: &'static Encoding,
        finalizing_chunk_received: bool,
    }

    impl Output {
        pub fn new(encoding: &'static Encoding) -> Self {
            Output {
                bytes: Vec::default(),
                encoding,
                finalizing_chunk_received: false,
            }
        }

        pub fn push(&mut self, chunk: &[u8]) {
            if chunk.is_empty() {
                self.finalizing_chunk_received = true;
            } else {
                assert!(
                    !self.finalizing_chunk_received,
                    "Chunk written to the output after the finalizing chunk."
                );

                self.bytes.extend_from_slice(chunk);
            }
        }
    }

    impl Into<String> for Output {
        fn into(self) -> String {
            assert!(
                self.finalizing_chunk_received,
                "Finalizing chunk for the output hasn't been received."
            );

            self.encoding
                .decode_without_bom_handling(&self.bytes)
                .0
                .into_owned()
        }
    }
}

cfg_if! {
    if #[cfg(feature = "integration_test")] {
        pub mod selectors_vm;

        pub use self::transform_stream::{
            StartTagHandlingResult, TransformController, TransformStream,
            TransformStreamSettings
        };

        pub use self::rewritable_units::{
            EndTag, Serialize, StartTag, Token, TokenCaptureFlags, Mutations
        };

        pub use self::base::Bytes;
        pub use self::memory::MemoryLimiter;
        pub use self::html::{LocalName, LocalNameHash, Tag, Namespace};
    } else {
        mod selectors_vm;
    }
}
