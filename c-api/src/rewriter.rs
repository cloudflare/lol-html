use super::*;

pub struct ExternOutputSink(extern "C" fn(*const c_char, size_t));

impl ExternOutputSink {
    fn new(sink: extern "C" fn(*const c_char, size_t)) -> Self {
        ExternOutputSink(sink)
    }
}

impl OutputSink for ExternOutputSink {
    #[inline]
    fn handle_chunk(&mut self, chunk: &[u8]) {
        self.0(chunk.as_ptr() as *const c_char, chunk.len());
    }
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_build(
    builder: *mut HtmlRewriterBuilder<'static>,
    encoding: *const c_char,
    encoding_len: size_t,
    output_sink: extern "C" fn(*const c_char, size_t),
) -> *mut HtmlRewriter<'static, ExternOutputSink> {
    let encoding = unwrap_or_ret_null! { to_str!(encoding, encoding_len) };
    let builder = to_box!(builder);

    let rewriter = unwrap_or_ret_null! {
        builder.build(encoding, ExternOutputSink::new(output_sink))
    };

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
