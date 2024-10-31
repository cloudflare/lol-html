use super::*;
use js_sys::{Function as JsFunction, Uint8Array};
use lol_html::{DocumentContentHandlers, ElementContentHandlers, OutputSink, Selector};

struct JsOutputSink(JsFunction);

impl JsOutputSink {
    fn new(func: &JsFunction) -> Self {
        Self(func.clone())
    }
}

impl OutputSink for JsOutputSink {
    #[inline]
    fn handle_chunk(&mut self, chunk: &[u8]) {
        let this = JsValue::NULL;
        let chunk = Uint8Array::from(chunk);

        // NOTE: the error is handled in the JS wrapper.
        self.0.call1(&this, &chunk).unwrap();
    }
}

#[wasm_bindgen]
pub struct HTMLRewriterBuilder {
    element_content_handlers: (Selector, ElementContentHandlers<'static>),
    document_content_handlers: DocumentContentHandlers<'static>,
}
