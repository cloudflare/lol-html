use super::rewriter_builder::HtmlRewriterBuilder;
use super::*;
use libc::c_void;
use std::convert::TryFrom;

// NOTE: we use `ExternOutputSink` proxy type, because we need an
// existential type parameter for the `HtmlRewriter` and FnMut can't
// be used as such since it's a trait.
pub struct ExternOutputSink {
    handler: unsafe extern "C" fn(*const c_char, size_t, *mut c_void),
    user_data: *mut c_void,
}

impl ExternOutputSink {
    #[inline]
    fn new(
        handler: unsafe extern "C" fn(*const c_char, size_t, *mut c_void),
        user_data: *mut c_void,
    ) -> Self {
        ExternOutputSink { handler, user_data }
    }
}

impl OutputSink for ExternOutputSink {
    #[inline]
    fn handle_chunk(&mut self, chunk: &[u8]) {
        let chunk_len = chunk.len();
        let chunk = chunk.as_ptr() as *const c_char;

        unsafe { (self.handler)(chunk, chunk_len, self.user_data) };
    }
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_build(
    builder: *mut HtmlRewriterBuilder,
    encoding: *const c_char,
    encoding_len: size_t,
    preallocated_memory: size_t,
    max_memory: size_t,
    output_sink: unsafe extern "C" fn(*const c_char, size_t, *mut c_void),
    output_sink_user_data: *mut c_void,
    strict: bool,
) -> *mut HtmlRewriter<'static, ExternOutputSink> {
    let builder = to_ref!(builder);
    let handlers = builder.get_safe_handlers();

    let settings = Settings {
        element_content_handlers: handlers.element,
        document_content_handlers: handlers.document,
        encoding: unwrap_or_ret_null! { to_str!(encoding, encoding_len) },
        max_memory,
        preallocated_memory,
        output_sink: ExternOutputSink::new(output_sink, output_sink_user_data),
        strict,
    };

    let rewriter = unwrap_or_ret_null! { HtmlRewriter::try_from(settings) };

    to_ptr_mut(rewriter)
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_write(
    rewriter: *mut HtmlRewriter<'static, ExternOutputSink>,
    chunk: *const c_char,
    chunk_len: size_t,
) -> c_int {
    let chunk = to_bytes!(chunk, chunk_len);
    let rewriter = to_ref_mut!(rewriter);

    unwrap_or_ret_err_code! { rewriter.write(chunk) };

    0
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_end(
    rewriter: *mut HtmlRewriter<'static, ExternOutputSink>,
) -> c_int {
    let rewriter = to_ref_mut!(rewriter);

    unwrap_or_ret_err_code! { rewriter.end() };

    0
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_free(rewriter: *mut HtmlRewriter<'static, ExternOutputSink>) {
    drop(to_box!(rewriter));
}
