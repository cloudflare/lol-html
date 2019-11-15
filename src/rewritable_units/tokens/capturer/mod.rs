mod text_decoder;
mod to_token;

use self::text_decoder::TextDecoder;
use super::*;
use crate::parser::Lexeme;
use crate::rewriter::{AsyncRewritingResult, RewritingResult};
use bitflags::bitflags;
use encoding_rs::Encoding;

pub use self::to_token::{ToToken, ToTokenResult};

bitflags! {
    pub struct TokenCaptureFlags: u8 {
        const TEXT = 0b0000_0001;
        const COMMENTS = 0b0000_0010;
        const NEXT_START_TAG = 0b0000_0100;
        const NEXT_END_TAG = 0b0000_1000;
        const DOCTYPES = 0b0001_0000;
    }
}

#[derive(Debug)]
pub enum TokenCapturerEvent<'i> {
    LexemeConsumed,
    TokenProduced(Box<Token<'i>>),
}

type CapturerEventHandler<'h, 'i> =
    &'h mut dyn FnMut(TokenCapturerEvent<'i>) -> AsyncRewritingResult;

pub struct TokenCapturer {
    encoding: &'static Encoding,
    text_decoder: TextDecoder,
    capture_flags: TokenCaptureFlags,
}

impl TokenCapturer {
    pub fn new(capture_flags: TokenCaptureFlags, encoding: &'static Encoding) -> Self {
        TokenCapturer {
            encoding,
            text_decoder: TextDecoder::new(encoding),
            capture_flags,
        }
    }

    #[inline]
    pub fn has_captures(&self) -> bool {
        !self.capture_flags.is_empty()
    }

    #[inline]
    pub fn set_capture_flags(&mut self, flags: TokenCaptureFlags) {
        self.capture_flags = flags;
    }

    #[inline]
    pub async fn flush_pending_text<'c>(
        &'c mut self,
        event_handler: CapturerEventHandler<'_, 'c>,
    ) -> RewritingResult {
        self.text_decoder.flush_pending(event_handler).await
    }

    pub async fn feed<'i, 'l, 'c: 'l, T>(
        &'c mut self,
        lexeme: &'l Lexeme<'i, T>,
        mut event_handler: impl FnMut(TokenCapturerEvent<'l>) -> AsyncRewritingResult,
    ) -> RewritingResult
    where
        Lexeme<'i, T>: ToToken,
    {
        match lexeme.to_token(&mut self.capture_flags, self.encoding) {
            ToTokenResult::Token(token) => {
                self.flush_pending_text(&mut event_handler).await?;
                event_handler(TokenCapturerEvent::LexemeConsumed).await?;
                event_handler(TokenCapturerEvent::TokenProduced(token)).await
            }
            ToTokenResult::Text(text_type) => {
                if self.capture_flags.contains(TokenCaptureFlags::TEXT) {
                    event_handler(TokenCapturerEvent::LexemeConsumed).await?;

                    self.text_decoder
                        .feed_text(&lexeme.raw(), text_type, &mut event_handler)
                        .await?;
                }

                Ok(())
            }
            ToTokenResult::None => self.flush_pending_text(&mut event_handler).await,
        }
    }
}
