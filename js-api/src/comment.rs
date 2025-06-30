use super::*;
use lol_html_native::html_content::Comment as NativeComment;

#[wasm_bindgen]
pub struct Comment(NativeRefWrap<NativeComment<'static>>);

impl_from_native!(NativeComment => Comment);
impl_mutations!(Comment);

#[wasm_bindgen]
impl Comment {
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> JsResult<String> {
        self.0.get().map(|c| c.text())
    }

    /// Returns a JS array `[start, end]` with byte offsets relative to the start of the document.
    ///
    /// The byte offsets are incompatible with JS's char code indices.
    #[wasm_bindgen(getter=sourceLocationBytes)]
    pub fn source_location_bytes(&self) -> JsResult<JsValue> {
        Ok(location_to_js(self.0.get()?.source_location()))
    }
}
