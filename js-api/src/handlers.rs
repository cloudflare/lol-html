use super::comment::Comment;
use super::doctype::Doctype;
use super::document_end::DocumentEnd;
use super::element::Element;
use super::text_chunk::TextChunk;
use super::*;
use js_sys::Function as JsFunction;
use lol_html_native::{
    DocumentContentHandlers as NativeDocumentContentHandlers,
    ElementContentHandlers as NativeElementContentHandlers,
};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("JS handler error")]
pub struct HandlerJsErrorWrap(pub JsValue);

// SAFETY: The exposed js-api only supports single-threaded usage.
unsafe impl Send for HandlerJsErrorWrap {}
unsafe impl Sync for HandlerJsErrorWrap {}

macro_rules! make_handler {
    ($handler:ident, $JsArgType:ident, $typehint:ty) => {{
        fn type_hint(h: $typehint) -> $typehint {
            h
        }
        type_hint(Box::new(move |arg: &mut _| {
            $JsArgType::with_native(arg, |js_value| $handler.call1(&JsValue::NULL, &js_value))
                .map_err(|e| HandlerJsErrorWrap(e))?;

            Ok(())
        }))
    }};
}

#[wasm_bindgen(typescript_custom_section)]
const INTERFACE_ELEMENT_CONTENT_HANDLERS: &'static str = r#"
export interface ElementContentHandlers {
  element?: (element: Element) => void;
  comments?: (comment: Comment) => void;
  text?: (textChunk: TextChunk) => void;
}"#;

#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(typescript_type = "ElementContentHandlers")]
    pub type ElementContentHandlers;

    #[wasm_bindgen(method, getter)]
    fn element(this: &ElementContentHandlers) -> Option<JsFunction>;

    #[wasm_bindgen(method, getter)]
    fn comments(this: &ElementContentHandlers) -> Option<JsFunction>;

    #[wasm_bindgen(method, getter)]
    fn text(this: &ElementContentHandlers) -> Option<JsFunction>;
}

impl IntoNative<NativeElementContentHandlers<'static>> for ElementContentHandlers {
    fn into_native(self) -> NativeElementContentHandlers<'static> {
        let mut native = NativeElementContentHandlers::default();

        if let Some(handler) = self.element() {
            native = native.element(make_handler!(
                handler,
                Element,
                lol_html_native::ElementHandler
            ));
        }

        if let Some(handler) = self.comments() {
            native = native.comments(make_handler!(
                handler,
                Comment,
                lol_html_native::CommentHandler
            ));
        }

        if let Some(handler) = self.text() {
            native = native.text(make_handler!(
                handler,
                TextChunk,
                lol_html_native::TextHandler
            ));
        }

        native
    }
}

#[wasm_bindgen(typescript_custom_section)]
const INTERFACE_DOCUMENT_CONTENT_HANDLERS: &'static str = r#"
export interface DocumentContentHandlers {
  doctype?: (doctype: Doctype) => void;
  comments?: (comment: Comment) => void;
  text?: (textChunk: TextChunk) => void;
  end?: (end: DocumentEnd) => void;
}"#;

#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(typescript_type = "DocumentContentHandlers")]
    pub type DocumentContentHandlers;

    #[wasm_bindgen(method, getter)]
    fn doctype(this: &DocumentContentHandlers) -> Option<JsFunction>;

    #[wasm_bindgen(method, getter)]
    fn comments(this: &DocumentContentHandlers) -> Option<JsFunction>;

    #[wasm_bindgen(method, getter)]
    fn text(this: &DocumentContentHandlers) -> Option<JsFunction>;

    #[wasm_bindgen(method, getter)]
    fn end(this: &DocumentContentHandlers) -> Option<JsFunction>;
}

impl IntoNative<NativeDocumentContentHandlers<'static>> for DocumentContentHandlers {
    fn into_native(self) -> NativeDocumentContentHandlers<'static> {
        let mut native = NativeDocumentContentHandlers::default();

        if let Some(handler) = self.doctype() {
            native = native.doctype(make_handler!(
                handler,
                Doctype,
                lol_html_native::DoctypeHandler
            ));
        }

        if let Some(handler) = self.comments() {
            native = native.comments(make_handler!(
                handler,
                Comment,
                lol_html_native::CommentHandler
            ));
        }

        if let Some(handler) = self.text() {
            native = native.text(make_handler!(
                handler,
                TextChunk,
                lol_html_native::TextHandler
            ));
        }

        if let Some(handler) = self.end() {
            native = native.end(make_handler!(
                handler,
                DocumentEnd,
                lol_html_native::EndHandler
            ));
        }

        native
    }
}
