use super::*;
use lol_html_native::html_content::Doctype as NativeDoctype;

#[wasm_bindgen]
pub struct Doctype(NativeRefWrap<NativeDoctype<'static>>);

impl_from_native!(NativeDoctype => Doctype);

#[wasm_bindgen]
impl Doctype {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> JsResult<Option<String>> {
        self.0.get().map(|d| d.name())
    }

    #[wasm_bindgen(getter=publicId)]
    pub fn public_id(&self) -> JsResult<Option<String>> {
        self.0.get().map(|d| d.public_id())
    }

    #[wasm_bindgen(getter=systemId)]
    pub fn system_id(&self) -> JsResult<Option<String>> {
        self.0.get().map(|d| d.system_id())
    }
}
