mod text_decoder;
mod to_token;

use self::text_decoder::TextDecoder;
use super::*;
use crate::base::SharedEncoding;
use crate::rewriter::RewritingError;
use bitflags::bitflags;

pub(crate) use self::to_token::{ToToken, ToTokenResult};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TokenCaptureFlags: u8 {
        const TEXT = 0b0000_0001;
        const COMMENTS = 0b0000_0010;
        const NEXT_START_TAG = 0b0000_0100;
        const NEXT_END_TAG = 0b0000_1000;
        const DOCTYPES = 0b0001_0000;
    }
}

#[derive(Debug)]
pub(crate) enum TokenCapturerEvent<'i> {
    LexemeConsumed,
    TokenProduced(Token<'i>),
}

pub(crate) type CapturerEventHandler<'h> =
    &'h mut dyn FnMut(TokenCapturerEvent<'_>) -> Result<(), RewritingError>;

pub(crate) struct TokenCapturer {
    pub encoding: SharedEncoding,
    pub text_decoder: TextDecoder,
    pub capture_flags: TokenCaptureFlags,
}

impl TokenCapturer {
    #[inline]
    #[must_use]
    pub fn new(capture_flags: TokenCaptureFlags, encoding: SharedEncoding) -> Self {
        Self {
            encoding: SharedEncoding::clone(&encoding),
            text_decoder: TextDecoder::new(encoding),
            capture_flags,
        }
    }

    #[inline]
    #[must_use]
    pub const fn has_captures(&self) -> bool {
        !self.capture_flags.is_empty()
    }

    #[inline]
    pub fn set_capture_flags(&mut self, flags: TokenCaptureFlags) {
        self.capture_flags = flags;
    }

    #[inline]
    pub fn flush_pending_text(
        &mut self,
        event_handler: CapturerEventHandler<'_>,
    ) -> Result<(), RewritingError> {
        self.text_decoder.flush_pending(event_handler)
    }


}
