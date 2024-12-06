use super::*;
use crate::errors::CStreamingHandlerError;
use lol_html::html_content::StreamingHandler;
use lol_html::html_content::StreamingHandlerSink;

/// Opaque type from C's perspective
pub type CStreamingHandlerSink<'tmp> = StreamingHandlerSink<'tmp>;

/// Write another piece of UTF-8 data to the output. Returns `0` on success, and `-1` if it wasn't valid UTF-8.
/// All pointers must be non-NULL.
#[no_mangle]
pub unsafe extern "C" fn lol_html_streaming_sink_write_str(
    sink: *mut CStreamingHandlerSink<'_>,
    string_utf8: *const c_char,
    string_utf8_len: size_t,
    is_html: bool,
) -> c_int {
    let sink = to_ref_mut!(sink);
    let content = unwrap_or_ret_err_code! { to_str!(string_utf8, string_utf8_len) };
    let is_html = if is_html {
        ContentType::Html
    } else {
        ContentType::Text
    };

    sink.write_str(content, is_html);
    0
}

/// [`StreamingHandlerSink::write_utf8_chunk`]
///
/// Writes as much of the given UTF-8 fragment as possible, converting the encoding and HTML-escaping if `is_html` is `false`.
///
/// The `bytes_utf8` doesn't need to be a complete UTF-8 string, as long as consecutive calls to this function create a valid UTF-8 string.
/// Any incomplete UTF-8 sequence at the end of the content is buffered and flushed as soon as it's completed.
///
/// Other functions like [`lol_html_streaming_sink_write_str`] should not be called after a
/// `lol_html_streaming_sink_write_utf8_chunk` call with an incomplete UTF-8 sequence.
///
/// Returns `0` on success, and `-1` if it wasn't valid UTF-8.
/// All pointers must be non-`NULL`.
#[no_mangle]
pub unsafe extern "C" fn lol_html_streaming_sink_write_utf8_chunk(
    sink: *mut CStreamingHandlerSink<'_>,
    bytes_utf8: *const c_char,
    bytes_utf8_len: size_t,
    is_html: bool,
) -> c_int {
    let sink = to_ref_mut!(sink);
    let content = to_bytes!(bytes_utf8, bytes_utf8_len);
    let is_html = if is_html {
        ContentType::Html
    } else {
        ContentType::Text
    };

    unwrap_or_ret_err_code! { sink.write_utf8_chunk(content, is_html) };
    0
}

/// Safety: the user data and the callbacks must be safe to use from a different thread (e.g. can't rely on thread-local storage).
///
/// It doesn't have to be `Sync`, it will be used only by one thread at a time.
///
/// Handler functions copy this struct. It can (and should) be created on the stack.
#[repr(C)]
pub struct CStreamingHandler {
    /// Anything you like
    pub user_data: *mut c_void,
    /// Called when the handler is supposed to produce its output. Return `0` for success.
    /// The `sink` argument is guaranteed non-`NULL`. It is valid only for the duration of this call, and can only be used on the same thread.
    /// The sink is for [`lol_html_streaming_sink_write_str`] and [`lol_html_streaming_sink_write_utf8_chunk`].
    /// `user_data` comes from this struct.
    /// `write_all_callback` must not be `NULL`.
    pub write_all_callback: Option<
        unsafe extern "C" fn(sink: &mut CStreamingHandlerSink<'_>, user_data: *mut c_void) -> c_int,
    >,
    /// Called exactly once, after the last use of this handler.
    /// `user_data` comes from this struct.
    /// May be `NULL`.
    pub drop_callback: Option<unsafe extern "C" fn(user_data: *mut c_void)>,
    /// *Always* initialize to `NULL`.
    pub reserved: *mut c_void,
}

// It's up to C to obey this
unsafe impl Send for CStreamingHandler {}

impl StreamingHandler for CStreamingHandler {
    fn write_all(
        self: Box<Self>,
        sink: &mut StreamingHandlerSink<'_>,
    ) -> Result<(), Box<(dyn std::error::Error + Send + Sync)>> {
        if !self.reserved.is_null() {
            return Err(CStreamingHandlerError::Uninitialized.into());
        }
        let cb = self
            .write_all_callback
            .ok_or(CStreamingHandlerError::Uninitialized)?;
        let res = unsafe { (cb)(sink, self.user_data) };
        if res == 0 {
            Ok(())
        } else {
            Err(CStreamingHandlerError::HandlerError(res).into())
        }
    }
}

impl Drop for CStreamingHandler {
    fn drop(&mut self) {
        if let Some(cb) = self.drop_callback {
            unsafe {
                cb(self.user_data);
            }
        }
    }
}
