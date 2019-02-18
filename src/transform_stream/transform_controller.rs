use crate::base::Chunk;
use crate::parser::{AttributeOultine, Lexeme, NextOutputType, TagHint, TokenOutline};
use crate::token::{
    Token, TokenCaptureFlags, CAPTURE_COMMENTS, CAPTURE_DOCTYPES, CAPTURE_END_TAGS,
    CAPTURE_START_TAGS, CAPTURE_TEXT,
};
use bitflags::bitflags;
use std::cell::RefCell;
use std::rc::Rc;

macro_rules! impl_into_token_capture_flags {
    ($Flags:ident) => {
        impl Into<TokenCaptureFlags> for $Flags {
            #[inline]
            fn into(self) -> TokenCaptureFlags {
                TokenCaptureFlags::from_bits_truncate(self.bits())
            }
        }
    };
}

bitflags! {
    pub struct DocumentLevelContentSettings: u8 {
        const TEXT = CAPTURE_TEXT;
        const COMMENTS = CAPTURE_COMMENTS;
        const DOCTYPES = CAPTURE_DOCTYPES;
    }
}

impl_into_token_capture_flags!(DocumentLevelContentSettings);

bitflags! {
    pub struct ContentSettingsOnElementStart: u8 {
        const CAPTURE_START_TAG_FOR_ELEMENT = CAPTURE_START_TAGS;
        const START_CAPTURING_TEXT = CAPTURE_TEXT;
        const START_CAPTURING_COMMENTS = CAPTURE_COMMENTS;
    }
}

impl_into_token_capture_flags!(ContentSettingsOnElementStart);

bitflags! {
    pub struct ContentSettingsOnElementEnd: u8 {
        const CAPTURE_END_TAG_FOR_ELEMENT = CAPTURE_END_TAGS;
        const STOP_CAPTURING_TEXT = !CAPTURE_TEXT;
        const STOP_CAPTURING_COMMENTS = !CAPTURE_COMMENTS;
    }
}

impl_into_token_capture_flags!(ContentSettingsOnElementEnd);

pub struct AttributesAndSelfClosingFlagInfo<'i> {
    input: &'i Chunk<'i>,
    attributes: Rc<RefCell<Vec<AttributeOultine>>>,
    self_closing: bool,
}

impl<'i> AttributesAndSelfClosingFlagInfo<'i> {
    #[inline]
    fn input(&self) -> &Chunk<'_> {
        self.input
    }

    #[inline]
    fn attributes(&self) -> &Rc<RefCell<Vec<AttributeOultine>>> {
        &self.attributes
    }

    #[inline]
    fn self_closing(&self) -> bool {
        self.self_closing
    }
}

impl<'i> From<&'i Lexeme<'i>> for AttributesAndSelfClosingFlagInfo<'i> {
    #[inline]
    fn from(lexeme: &'i Lexeme<'i>) -> Self {
        match lexeme.token_outline() {
            Some(TokenOutline::StartTag {
                attributes,
                self_closing,
                ..
            }) => AttributesAndSelfClosingFlagInfo {
                input: lexeme.input(),
                attributes: Rc::clone(attributes),
                self_closing: *self_closing,
            },
            _ => unreachable!("Lexeme should be a start tag"),
        }
    }
}

pub enum ElementStartResponse<'h> {
    ContentSettings(ContentSettingsOnElementStart),
    RequestFullStartTagInfo(Box<FnMut(AttributesAndSelfClosingFlagInfo) + 'h>),
}

pub trait TransformController {
    fn document_level_content_settings(&self) -> DocumentLevelContentSettings;
    fn get_initial_token_capture_flags(&self) -> TokenCaptureFlags;
    fn get_token_capture_flags_for_tag(&mut self, tag_lexeme: &Lexeme<'_>) -> NextOutputType;
    fn get_token_capture_flags_for_tag_hint(&mut self, tag_hint: &TagHint<'_>) -> NextOutputType;
    fn handle_token(&mut self, token: &mut Token<'_>);
}
