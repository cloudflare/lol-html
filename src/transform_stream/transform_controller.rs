use crate::base::Chunk;
use crate::html::LocalName;
use crate::parser::SharedAttributeBuffer;
use crate::rewritable_units::{Token, TokenCaptureFlags};

pub struct AuxiliaryElementInfo<'i> {
    input: &'i Chunk<'i>,
    attributes: SharedAttributeBuffer,
    self_closing: bool,
}

impl<'i> AuxiliaryElementInfo<'i> {
    #[inline]
    pub fn new(
        input: &'i Chunk<'i>,
        attributes: SharedAttributeBuffer,
        self_closing: bool,
    ) -> Self {
        AuxiliaryElementInfo {
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

pub type AuxiliaryElementInfoHandler<C> =
    Box<dyn FnMut(&mut C, AuxiliaryElementInfo) -> TokenCaptureFlags>;

pub enum ElementStartResponse<C: TransformController> {
    CaptureFlags(TokenCaptureFlags),
    RequestAuxiliaryElementInfo(AuxiliaryElementInfoHandler<C>),
}

pub trait TransformController: Sized {
    fn initial_capture_flags(&self) -> TokenCaptureFlags;
    fn handle_element_start(&mut self, name: LocalName<'_>) -> ElementStartResponse<Self>;
    fn handle_element_end(&mut self, name: LocalName<'_>) -> TokenCaptureFlags;
    fn handle_token(&mut self, token: &mut Token<'_>);
}
