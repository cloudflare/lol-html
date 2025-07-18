use super::end_tag::EndTag;
use super::*;
use js_sys::Function as JsFunction;
use lol_html_native::html_content::{Attribute as NativeAttribute, Element as NativeElement};
use serde::Serialize;
use serde_wasm_bindgen::to_value as to_js_value;
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

#[derive(Serialize)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

impl From<&NativeAttribute<'_>> for Attribute {
    fn from(native: &NativeAttribute) -> Self {
        Self {
            name: native.name(),
            value: native.value(),
        }
    }
}

#[wasm_bindgen]
pub struct Element(NativeRefWrap<NativeElement<'static, 'static>>);

impl_from_native!(NativeElement => Element);
impl_mutations!(Element);

#[wasm_bindgen]
impl Element {
    #[wasm_bindgen(getter=tagName)]
    pub fn tag_name(&self) -> JsResult<String> {
        self.0.get().map(|e| e.tag_name())
    }

    #[wasm_bindgen(setter=tagName)]
    pub fn set_tag_name(&mut self, name: &str) -> JsResult<()> {
        self.0.get_mut()?.set_tag_name(name).into_js_result()
    }

    #[wasm_bindgen(getter=namespaceURI)]
    pub fn namespace_uri(&self) -> JsResult<JsValue> {
        self.0.get().map(|e| e.namespace_uri().into())
    }

    #[wasm_bindgen(getter)]
    pub fn attributes(&self) -> JsResult<JsValue> {
        self.0
            .get()
            .map(|e| {
                e.attributes()
                    .iter()
                    .map(Attribute::from)
                    .collect::<Vec<_>>()
            })
            .and_then(|a| to_js_value(&a).into_js_result())
    }

    #[wasm_bindgen(js_name=getAttribute)]
    pub fn get_attribute(&self, name: &str) -> JsResult<Option<String>> {
        self.0.get().map(|e| e.get_attribute(name))
    }

    #[wasm_bindgen(js_name=hasAttribute)]
    pub fn has_attribute(&self, name: &str) -> JsResult<bool> {
        self.0.get().map(|e| e.has_attribute(name))
    }

    #[wasm_bindgen(js_name=setAttribute)]
    pub fn set_attribute(&mut self, name: &str, value: &str) -> JsResult<()> {
        self.0
            .get_mut()?
            .set_attribute(name, value)
            .into_js_result()
    }

    #[wasm_bindgen(js_name=removeAttribute)]
    pub fn remove_attribute(&mut self, name: &str) -> JsResult<()> {
        self.0.get_mut().map(|e| e.remove_attribute(name))
    }

    pub fn prepend(
        &mut self,
        content: &str,
        content_type: Option<ContentTypeOptions>,
    ) -> Result<(), JsValue> {
        self.0
            .get_mut()
            .map(|e| e.prepend(content, content_type.into_native()))
    }

    pub fn append(
        &mut self,
        content: &str,
        content_type: Option<ContentTypeOptions>,
    ) -> Result<(), JsValue> {
        self.0
            .get_mut()
            .map(|e| e.append(content, content_type.into_native()))
    }

    #[wasm_bindgen(js_name=setInnerContent)]
    pub fn set_inner_content(
        &mut self,
        content: &str,
        content_type: Option<ContentTypeOptions>,
    ) -> Result<(), JsValue> {
        self.0
            .get_mut()
            .map(|e| e.set_inner_content(content, content_type.into_native()))
    }

    #[wasm_bindgen(js_name=removeAndKeepContent)]
    pub fn remove_and_keep_content(&mut self) -> Result<(), JsValue> {
        self.0.get_mut().map(|e| e.remove_and_keep_content())
    }

    #[wasm_bindgen(js_name=onEndTag)]
    pub fn on_end_tag(&mut self, handler: JsFunction) -> JsResult<()> {
        if let Some(handlers) = self.0.get_mut()?.end_tag_handlers() {
            handlers.push(make_handler!(handler, EndTag, lol_html_native::EndTagHandler));
        }

        Ok(())
    }

    /// Returns a JS array `[start, end]` with byte offsets relative to the start of the document.
    /// The location is for the start tag only.
    ///
    /// The byte offsets are incompatible with JS's char code indices.
    #[wasm_bindgen(getter=sourceLocationBytes)]
    pub fn source_location_bytes(&self) -> JsResult<JsValue> {
        Ok(location_to_js(self.0.get()?.source_location()))
    }
}
