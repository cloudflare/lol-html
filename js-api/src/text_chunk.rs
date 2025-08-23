use super::*;
use lol_html_native::html_content::TextChunk as NativeTextChunk;

#[wasm_bindgen]
pub struct TextChunk(NativeRefWrap<NativeTextChunk<'static>>);

impl_from_native!(NativeTextChunk => TextChunk);
impl_mutations!(TextChunk);

#[wasm_bindgen]
impl TextChunk {
    /// The text may be an incomplete fragment of a text node
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> JsResult<String> {
        self.0.get().map(|c| c.as_str().into())
    }

    #[wasm_bindgen(getter=lastInTextNode)]
    pub fn last_in_text_node(&self) -> JsResult<bool> {
        self.0.get().map(|c| c.last_in_text_node())
    }

    /// Returns a JS array `[start, end]` with byte offsets relative to the start of the document.
    ///
    /// The byte offsets are incompatible with JS's char code indices.
    #[wasm_bindgen(getter=sourceLocationBytes, unchecked_return_type="[number, number]")]
    pub fn source_location_bytes(&self) -> JsResult<JsValue> {
        Ok(location_to_js(self.0.get()?.source_location()))
    }
}
