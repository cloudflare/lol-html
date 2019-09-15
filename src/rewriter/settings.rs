use crate::rewritable_units::{Comment, Doctype, Element, EndTag, TextChunk};
use crate::selectors_vm::Selector;
use failure::Error;

pub type DoctypeHandler<'h> = Box<dyn FnMut(&mut Doctype) -> Result<(), Error> + 'h>;
pub type CommentHandler<'h> = Box<dyn FnMut(&mut Comment) -> Result<(), Error> + 'h>;
pub type TextHandler<'h> = Box<dyn FnMut(&mut TextChunk) -> Result<(), Error> + 'h>;
pub type ElementHandler<'h> = Box<dyn FnMut(&mut Element) -> Result<(), Error> + 'h>;
pub type EndTagHandler<'h> = Box<dyn FnMut(&mut EndTag) -> Result<(), Error> + 'h>;

#[derive(Default)]
pub struct ElementContentHandlers<'h> {
    pub(super) element: Option<ElementHandler<'h>>,
    pub(super) comments: Option<CommentHandler<'h>>,
    pub(super) text: Option<TextHandler<'h>>,
}

impl<'h> ElementContentHandlers<'h> {
    #[inline]
    pub fn element(mut self, handler: impl FnMut(&mut Element) -> Result<(), Error> + 'h) -> Self {
        self.element = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn comments(mut self, handler: impl FnMut(&mut Comment) -> Result<(), Error> + 'h) -> Self {
        self.comments = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn text(mut self, handler: impl FnMut(&mut TextChunk) -> Result<(), Error> + 'h) -> Self {
        self.text = Some(Box::new(handler));

        self
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __element_content_handler {
    ($selector:expr, $handler_name:ident, $handler:expr) => {
        (
            &$selector.parse::<$crate::Selector>().unwrap(),
            $crate::ElementContentHandlers::default().$handler_name($handler),
        )
    };
}

#[macro_export(local_inner_macros)]
macro_rules! element {
    ($selector:expr, $handler:expr) => {
        __element_content_handler!($selector, element, $handler);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! text {
    ($selector:expr, $handler:expr) => {
        __element_content_handler!($selector, text, $handler);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! comments {
    ($selector:expr, $handler:expr) => {
        __element_content_handler!($selector, comments, $handler);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __document_content_handler {
    ($handler_name:ident, $handler:expr) => {
        $crate::DocumentContentHandlers::default().$handler_name($handler)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! doctype {
    ($handler:expr) => {
        __document_content_handler!(doctype, $handler);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! doc_text {
    ($handler:expr) => {
        __document_content_handler!(text, $handler);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! doc_comments {
    ($handler:expr) => {
        __document_content_handler!(comments, $handler);
    };
}

#[derive(Default)]
pub struct DocumentContentHandlers<'h> {
    pub(super) doctype: Option<DoctypeHandler<'h>>,
    pub(super) comments: Option<CommentHandler<'h>>,
    pub(super) text: Option<TextHandler<'h>>,
}

impl<'h> DocumentContentHandlers<'h> {
    #[inline]
    pub fn doctype(mut self, handler: impl FnMut(&mut Doctype) -> Result<(), Error> + 'h) -> Self {
        self.doctype = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn comments(mut self, handler: impl FnMut(&mut Comment) -> Result<(), Error> + 'h) -> Self {
        self.comments = Some(Box::new(handler));

        self
    }

    #[inline]
    pub fn text(mut self, handler: impl FnMut(&mut TextChunk) -> Result<(), Error> + 'h) -> Self {
        self.text = Some(Box::new(handler));

        self
    }
}

// NOTE: exposed in C API as well, thus repr(C).
#[repr(C)]
pub struct MemorySettings {
    pub preallocated_parsing_buffer_size: usize,
    pub max_allowed_memory_usage: usize,
}

impl Default for MemorySettings {
    #[inline]
    fn default() -> Self {
        MemorySettings {
            preallocated_parsing_buffer_size: 1024,
            max_allowed_memory_usage: std::usize::MAX,
        }
    }
}

pub struct Settings<'h, 's> {
    pub element_content_handlers: Vec<(&'s Selector, ElementContentHandlers<'h>)>,
    pub document_content_handlers: Vec<DocumentContentHandlers<'h>>,
    pub encoding: &'s str,
    pub memory_settings: MemorySettings,
    pub strict: bool,
}

impl Default for Settings<'_, '_> {
    #[inline]
    fn default() -> Self {
        Settings {
            element_content_handlers: vec![],
            document_content_handlers: vec![],
            encoding: "utf-8",
            memory_settings: MemorySettings::default(),
            strict: true,
        }
    }
}
