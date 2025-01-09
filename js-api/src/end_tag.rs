use super::*;
use lol_html::html_content::EndTag as NativeEndTag;

#[wasm_bindgen]
pub struct EndTag(NativeRefWrap<NativeEndTag<'static>>);

impl_from_native!(NativeEndTag => EndTag);
impl_mutations_end_tag!(EndTag);

#[wasm_bindgen]
impl EndTag {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> JsResult<String> {
        self.0.get().map(|d| d.name())
    }
}
