use crate::base::Chunk;
use crate::content::{
    Token, TokenCaptureFlags, CAPTURE_COMMENTS, CAPTURE_DOCTYPES, CAPTURE_END_TAGS,
    CAPTURE_START_TAGS, CAPTURE_TEXT,
};
use crate::parser::{SharedAttributeBuffer, TagNameInfo};
use bitflags::bitflags;

macro_rules! impl_into_token_capturer_flags {
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
    #[derive(Default)]
    pub struct DocumentLevelContentSettings: u8 {
        const CAPTURE_TEXT = CAPTURE_TEXT;
        const CAPTURE_COMMENTS = CAPTURE_COMMENTS;
        const CAPTURE_DOCTYPES = CAPTURE_DOCTYPES;
    }
}

impl_into_token_capturer_flags!(DocumentLevelContentSettings);

bitflags! {
    pub struct ContentSettingsOnElementStart: u8 {
        const CAPTURE_START_TAG_FOR_ELEMENT = CAPTURE_START_TAGS;
        const CAPTURE_TEXT = CAPTURE_TEXT;
        const CAPTURE_COMMENTS = CAPTURE_COMMENTS;
    }
}

impl_into_token_capturer_flags!(ContentSettingsOnElementStart);

bitflags! {
    pub struct ContentSettingsOnElementEnd: u8 {
        const CAPTURE_END_TAG_FOR_ELEMENT = CAPTURE_END_TAGS;
        const CAPTURE_TEXT = CAPTURE_TEXT;
        const CAPTURE_COMMENTS = CAPTURE_COMMENTS;
    }
}

impl_into_token_capturer_flags!(ContentSettingsOnElementEnd);

pub struct ElementModifiersInfo<'i> {
    input: &'i Chunk<'i>,
    attributes: SharedAttributeBuffer,
    self_closing: bool,
}

impl<'i> ElementModifiersInfo<'i> {
    #[inline]
    pub fn new(
        input: &'i Chunk<'i>,
        attributes: SharedAttributeBuffer,
        self_closing: bool,
    ) -> Self {
        ElementModifiersInfo {
            input,
            attributes,
            self_closing,
        }
    }
    #[inline]
    pub fn input(&self) -> &Chunk<'_> {
        self.input
    }

    #[inline]
    pub fn attributes(&self) -> &SharedAttributeBuffer {
        &self.attributes
    }

    #[inline]
    pub fn self_closing(&self) -> bool {
        self.self_closing
    }
}

pub type ElementModifiersInfoHandler<C> =
    Box<dyn FnMut(&mut C, ElementModifiersInfo) -> ContentSettingsOnElementStart>;

pub enum ElementStartResponse<C: TransformController> {
    ContentSettings(ContentSettingsOnElementStart),
    RequestElementModifiersInfo(ElementModifiersInfoHandler<C>),
}

pub trait TransformController: Sized {
    fn document_level_content_settings(&self) -> DocumentLevelContentSettings;
    fn handle_element_start(&mut self, name_info: &TagNameInfo<'_>) -> ElementStartResponse<Self>;
    fn handle_element_end(&mut self, name_info: &TagNameInfo<'_>) -> ContentSettingsOnElementEnd;
    fn handle_token(&mut self, token: &mut Token<'_>);
}
