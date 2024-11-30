use super::*;
use crate::html::TextType;
use crate::parser::{NonTagContentLexeme, NonTagContentTokenOutline, TagLexeme, TagTokenOutline};
use encoding_rs::Encoding;

pub(crate) enum ToTokenResult<'i> {
    Token(Token<'i>),
    Text(TextType),
    None,
}

pub(crate) trait ToToken {
    fn to_token<'s>(
        &'s self,
        capture_flags: &mut TokenCaptureFlags,
        encoding: &'static Encoding,
        out: &mut ToTokenResult<'s>,
    );
}

impl<'i> ToToken for TagLexeme<'i> {
    fn to_token<'x>(
        &'x self,
        capture_flags: &mut TokenCaptureFlags,
        encoding: &'static Encoding,
        out: &mut ToTokenResult<'x>,
    ) {
        debug_assert!(matches!(out, ToTokenResult::None));

        match *self.token_outline() {
            TagTokenOutline::StartTag {
                name,
                ref attributes,
                ns,
                self_closing,
                ..
            } => {
                if capture_flags.contains(TokenCaptureFlags::NEXT_START_TAG) {
                    // NOTE: clear the flag once we've seen required start tag.
                    capture_flags.remove(TokenCaptureFlags::NEXT_START_TAG);
                    *out = ToTokenResult::Token(StartTag::new_token(
                        self.part(name),
                        Attributes::new(self.input(), attributes, encoding),
                        ns,
                        self_closing,
                        self.raw(),
                        encoding,
                    ));
                }
            }

            TagTokenOutline::EndTag { name, .. } => {
                if capture_flags.contains(TokenCaptureFlags::NEXT_END_TAG) {
                    // NOTE: clear the flag once we've seen required end tag.
                    capture_flags.remove(TokenCaptureFlags::NEXT_END_TAG);
                    *out = ToTokenResult::Token(EndTag::new_token(
                        self.part(name),
                        self.raw(),
                        encoding,
                    ))
                }
            }
        }
    }
}

impl ToToken for NonTagContentLexeme<'_> {
    fn to_token<'s>(
        &'s self,
        capture_flags: &mut TokenCaptureFlags,
        encoding: &'static Encoding,
        out: &mut ToTokenResult<'s>,
    ) {
        debug_assert!(matches!(out, ToTokenResult::None));

        match *self.token_outline() {
            Some(NonTagContentTokenOutline::Text(text_type)) => {
                *out = ToTokenResult::Text(text_type)
            }
            Some(NonTagContentTokenOutline::Comment(text)) => {
                if capture_flags.contains(TokenCaptureFlags::COMMENTS) {
                    *out = ToTokenResult::Token(Comment::new_token(
                        self.part(text),
                        self.raw(),
                        encoding,
                    ))
                }
            }

            Some(NonTagContentTokenOutline::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            }) => {
                if capture_flags.contains(TokenCaptureFlags::DOCTYPES) {
                    *out = ToTokenResult::Token(Doctype::new_token(
                        self.opt_part(name),
                        self.opt_part(public_id),
                        self.opt_part(system_id),
                        force_quirks,
                        false, // removed
                        self.raw(),
                        encoding,
                    ))
                }
            }
            Some(NonTagContentTokenOutline::Eof) | None => {}
        }
    }
}
