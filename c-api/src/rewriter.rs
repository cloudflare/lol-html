use super::*;
use libc::c_void;

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
    builder: *mut HtmlRewriterBuilder<'static>,
    encoding: *const c_char,
    encoding_len: size_t,
    output_sink: unsafe extern "C" fn(*const c_char, size_t, *mut c_void),
    output_sink_user_data: *mut c_void,
) -> *mut HtmlRewriter<'static, ExternOutputSink> {
    let encoding = unwrap_or_ret_null! { to_str!(encoding, encoding_len) };
    let builder = to_ref!(builder);
    let output_sink = ExternOutputSink::new(output_sink, output_sink_user_data);
    let rewriter = unwrap_or_ret_null! { builder.build(encoding, output_sink) };

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
