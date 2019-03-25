use crate::base::Chunk;
use crate::content::{Token, TokenCaptureFlags};
use crate::parser::{SharedAttributeBuffer, TagNameInfo};

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
    Box<dyn FnMut(&mut C, ElementModifiersInfo) -> TokenCaptureFlags>;

pub enum ElementStartResponse<C: TransformController> {
    CaptureFlags(TokenCaptureFlags),
    RequestElementModifiersInfo(ElementModifiersInfoHandler<C>),
}

pub trait TransformController: Sized {
    fn initial_capture_flags(&self) -> TokenCaptureFlags;
    fn handle_element_start(&mut self, name_info: &TagNameInfo<'_>) -> ElementStartResponse<Self>;
    fn handle_element_end(&mut self, name_info: &TagNameInfo<'_>) -> TokenCaptureFlags;
    fn handle_token(&mut self, token: &mut Token<'_>);
}
