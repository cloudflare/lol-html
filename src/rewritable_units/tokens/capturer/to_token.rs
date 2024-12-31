use super::TokenCaptureFlags;
use crate::html::TextType;
use crate::parser::{NonTagContentLexeme, NonTagContentTokenOutline, TagLexeme, TagTokenOutline};
use crate::rewritable_units::{Attributes, Comment, Doctype, EndTag, StartTag, Token};
use encoding_rs::Encoding;

pub(crate) enum ToTokenResult<'i> {
    Token(Token<'i>),
    Text(TextType),
    None,
}

pub(crate) trait ToToken {
    fn to_token(
        &self,
        capture_flags: &mut TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> ToTokenResult<'_>;
}

impl<'i> ToToken for TagLexeme<'i> {
    #[inline]
    fn to_token(
        &self,
        capture_flags: &mut TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> ToTokenResult<'_> {
        match *self.token_outline() {
            TagTokenOutline::StartTag {
                name,
                ref attributes,
                ns,
                self_closing,
                ..
            } if capture_flags.contains(TokenCaptureFlags::NEXT_START_TAG) => {
                // NOTE: clear the flag once we've seen required start tag.
                capture_flags.remove(TokenCaptureFlags::NEXT_START_TAG);
                ToTokenResult::Token(StartTag::new_token(
                    self.part(name),
                    Attributes::new(self.input(), attributes, encoding),
                    ns,
                    self_closing,
                    self.raw(),
                    encoding,
                ))
            }

            TagTokenOutline::EndTag { name, .. }
                if capture_flags.contains(TokenCaptureFlags::NEXT_END_TAG) =>
            {
                // NOTE: clear the flag once we've seen required end tag.
                capture_flags.remove(TokenCaptureFlags::NEXT_END_TAG);
                ToTokenResult::Token(EndTag::new_token(self.part(name), self.raw(), encoding))
            }
            _ => ToTokenResult::None,
        }
    }
}

impl ToToken for NonTagContentLexeme<'_> {
    #[inline]
    fn to_token(
        &self,
        capture_flags: &mut TokenCaptureFlags,
        encoding: &'static Encoding,
    ) -> ToTokenResult<'_> {
        match *self.token_outline() {
            Some(NonTagContentTokenOutline::Text(text_type))
                if capture_flags.contains(TokenCaptureFlags::TEXT) =>
            {
                ToTokenResult::Text(text_type)
            }

            Some(NonTagContentTokenOutline::Comment(text))
                if capture_flags.contains(TokenCaptureFlags::COMMENTS) =>
            {
                ToTokenResult::Token(Comment::new_token(self.part(text), self.raw(), encoding))
            }

            Some(NonTagContentTokenOutline::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            }) if capture_flags.contains(TokenCaptureFlags::DOCTYPES) => {
                ToTokenResult::Token(Doctype::new_token(
                    self.opt_part(name),
                    self.opt_part(public_id),
                    self.opt_part(system_id),
                    force_quirks,
                    false, // removed
                    self.raw(),
                    encoding,
                ))
            }
            _ => ToTokenResult::None,
        }
    }
}
