use super::handlers::{DocumentContentHandlers, ElementContentHandlers, HandlerJsErrorWrap};
use super::*;
use encoding_rs::Encoding;
use js_sys::{Error as JsError, Function as JsFunction, Uint8Array};
use lol_html_native::errors::RewritingError;
use lol_html_native::{
    AsciiCompatibleEncoding, HtmlRewriter as NativeHTMLRewriter, OutputSink, Selector, Settings,
};
use std::borrow::Cow;

fn map_err(err: RewritingError) -> JsValue {
    match err {
        RewritingError::ContentHandlerError(err) => err.downcast::<HandlerJsErrorWrap>().unwrap().0,
        _ => JsValue::from(err.to_string()),
    }
}

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

#[allow(clippy::large_enum_variant)]
enum RewriterState {
    Before {
        settings: Settings<'static, 'static>,
        output_sink: JsOutputSink,
    },
    During(NativeHTMLRewriter<'static, JsOutputSink>),
    After,
}

#[wasm_bindgen]
pub struct HTMLRewriter(RewriterState);

#[wasm_bindgen]
impl HTMLRewriter {
    #[wasm_bindgen(constructor)]
    pub fn new(
        encoding: &str,
        #[wasm_bindgen(unchecked_param_type = "(chunk: Uint8Array) => void")]
        output_sink: &JsFunction,
    ) -> JsResult<Self> {
        let encoding = Encoding::for_label(encoding.as_bytes())
            .and_then(AsciiCompatibleEncoding::new)
            .ok_or_else(|| JsError::new("Invalid encoding"))?;

        Ok(Self(RewriterState::Before {
            output_sink: JsOutputSink::new(output_sink),
            settings: Settings {
                encoding,
                // TODO: accept options bag and parse out here
                ..Settings::default()
            },
        }))
    }

    fn inner_mut(&mut self) -> JsResult<&mut NativeHTMLRewriter<'static, JsOutputSink>> {
        match &mut self.0 {
            RewriterState::During(inner) => Ok(inner),
            state => Self::advance_state(std::mem::replace(state, RewriterState::After), state),
        }
    }

    #[cold]
    fn advance_state(
        old_state: RewriterState,
        new_state: &mut RewriterState,
    ) -> JsResult<&mut NativeHTMLRewriter<'static, JsOutputSink>> {
        if let RewriterState::Before {
            settings,
            output_sink,
        } = old_state
        {
            *new_state = RewriterState::During(NativeHTMLRewriter::new(settings, output_sink));
            if let RewriterState::During(inner) = new_state {
                return Ok(inner);
            }
        }

        Err(JsError::new("Rewriter is ended").into())
    }

    pub fn on(&mut self, selector: &str, handlers: ElementContentHandlers) -> JsResult<()> {
        match self.0 {
            RewriterState::Before {
                ref mut settings, ..
            } => {
                let selector = selector.parse::<Selector>().into_js_result()?;

                settings
                    .element_content_handlers
                    .push((Cow::Owned(selector), handlers.into_native()));

                Ok(())
            }
            _ => Err(JsError::new("Handlers cannot be added after write").into()),
        }
    }

    #[wasm_bindgen(js_name=onDocument)]
    pub fn on_document(&mut self, handlers: DocumentContentHandlers) -> JsResult<()> {
        match self.0 {
            RewriterState::Before {
                ref mut settings, ..
            } => {
                settings
                    .document_content_handlers
                    .push(handlers.into_native());
                Ok(())
            }
            _ => Err(JsError::new("Handlers cannot be added after write").into()),
        }
    }

    pub fn write(&mut self, chunk: &[u8]) -> JsResult<()> {
        self.inner_mut()?.write(chunk).map_err(map_err)
    }

    pub fn end(&mut self) -> JsResult<()> {
        match std::mem::replace(&mut self.0, RewriterState::After) {
            RewriterState::During(inner) => inner.end().map_err(map_err),
            _ => Ok(()),
        }
    }
}
