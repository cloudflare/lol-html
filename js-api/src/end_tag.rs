use super::*;
use lol_html_native::html_content::EndTag as NativeEndTag;

#[wasm_bindgen]
pub struct EndTag(NativeRefWrap<NativeEndTag<'static>>);

impl_from_native!(NativeEndTag => EndTag);
impl_mutations!(EndTag);

#[wasm_bindgen]
impl EndTag {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> JsResult<String> {
        self.0.get().map(|d| d.name())
    }

    /// Returns a JS array `[start, end]` with byte offsets relative to the start of the document.
    ///
    /// The byte offsets are incompatible with JS's char code indices.
    #[wasm_bindgen(getter=sourceLocationBytes)]
    pub fn source_location_bytes(&self) -> JsResult<JsValue> {
        Ok(location_to_js(self.0.get()?.source_location()))
    }
}
