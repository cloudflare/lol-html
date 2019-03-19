use cool_thing::*;
use failure::Error;
use libc::{c_char, c_int, size_t};
use std::cell::RefCell;
use std::ops::Drop;
use std::ptr;
use std::slice;
use std::str;

thread_local! {
    static LAST_ERROR: RefCell<Option<Error>> = RefCell::new(None);
}

// NOTE: we don't use CStr and CString as the transfer type because UTF8
// string comming from both sides can contain interior NULLs.
#[repr(C)]
pub struct Str {
    data: *const c_char,
    len: size_t,
}

impl Str {
    #[inline]
    fn ptr(string: String) -> *const Self {
        let len = string.len();
        let bytes = string.into_boxed_str().into_boxed_bytes();

        let string = Str {
            data: Box::into_raw(bytes) as *const c_char,
            len,
        };

        Box::into_raw(Box::new(string))
    }

    #[inline]
    fn opt_ptr(string: Option<String>) -> *const Self {
        match string {
            Some(string) => Self::ptr(string),
            None => ptr::null(),
        }
    }
}

impl Drop for Str {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.data as *mut c_char) });
    }
}

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

#[inline]
fn bytes_from_raw(data: *const c_char, len: size_t) -> &'static [u8] {
    unsafe { slice::from_raw_parts(data as *const u8, len) }
}

#[inline]
fn str_from_raw(data: *const c_char, len: size_t) -> Result<&'static str, Error> {
    let bytes = bytes_from_raw(data, len);

    str::from_utf8(bytes).map_err(Error::from)
}

// NOTE: abort the thread if we receive NULL where unexpected
macro_rules! assert_not_null {
    ($var:ident) => {
        assert!(!$var.is_null(), "{} is NULL", stringify!($var));
    };
}

macro_rules! safe_unwrap {
    ($expr:expr, $ret_val:expr) => {
        match $expr {
            Ok(v) => v,
            Err(err) => {
                LAST_ERROR.with(|e| *e.borrow_mut() = Some(err.into()));
                return $ret_val;
            }
        }
    };
}

macro_rules! unwrap_or_code {
    ($expr:expr) => {
        safe_unwrap!($expr, -1)
    };
}

macro_rules! unwrap_or_null {
    ($expr:expr) => {
        safe_unwrap!($expr, ptr::null_mut())
    };
}

#[no_mangle]
pub extern "C" fn cool_thing_str_free(string: *mut Str) {
    assert_not_null!(string);

    drop(unsafe { Box::from_raw(string as *mut Str) });
}

#[no_mangle]
pub extern "C" fn cool_thing_get_last_error() -> *const Str {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map_or(ptr::null(), |e| Str::ptr(e.to_string()))
    })
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_new() -> *mut HtmlRewriterBuilder<'static> {
    Box::into_raw(Box::new(HtmlRewriterBuilder::default()))
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_add_document_content_handlers(
    builder: *mut HtmlRewriterBuilder<'static>,
    doctype_handler: Option<extern "C" fn(*mut Doctype<'_>)>,
    comments_handler: Option<extern "C" fn(*mut Comment<'_>)>,
    text_handler: Option<extern "C" fn(*mut TextChunk<'_>)>,
) {
    assert_not_null!(builder);

    let builder = unsafe { &mut *builder };
    let mut handlers = DocumentContentHandlers::default();

    if let Some(handler) = doctype_handler {
        handlers = handlers.doctype(move |d| handler(d));
    }

    if let Some(handler) = comments_handler {
        handlers = handlers.comments(move |c| handler(c));
    }

    if let Some(handler) = text_handler {
        handlers = handlers.text(move |c| handler(c));
    }

    builder.on_document(handlers);
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_builder_add_element_content_handlers(
    builder: *mut HtmlRewriterBuilder<'static>,
    selector: *const c_char,
    selector_len: size_t,
    element_handler: Option<extern "C" fn(*mut Element<'_, '_>)>,
    comments_handler: Option<extern "C" fn(*mut Comment<'_>)>,
    text_handler: Option<extern "C" fn(*mut TextChunk<'_>)>,
) -> c_int {
    assert_not_null!(builder);
    assert_not_null!(selector);

    let selector = unwrap_or_code! { str_from_raw(selector, selector_len) };
    let builder = unsafe { &mut *builder };
    let mut handlers = ElementContentHandlers::default();

    if let Some(handler) = element_handler {
        handlers = handlers.element(move |e| handler(e));
    }

    if let Some(handler) = comments_handler {
        handlers = handlers.comments(move |c| handler(c));
    }

    if let Some(handler) = text_handler {
        handlers = handlers.text(move |c| handler(c));
    }

    unwrap_or_code! { builder.on(selector, handlers) };

    0
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_build(
    builder: *mut HtmlRewriterBuilder<'static>,
    encoding: *const c_char,
    encoding_len: size_t,
    output_sink: extern "C" fn(*const c_char, size_t),
) -> *mut HtmlRewriter<'static, ExternOutputSink> {
    assert_not_null!(builder);
    assert_not_null!(encoding);

    let encoding = unwrap_or_null! { str_from_raw(encoding, encoding_len) };
    let builder = unsafe { Box::from_raw(builder) };

    let rewriter = unwrap_or_null! {
        builder.build(encoding, ExternOutputSink::new(output_sink))
    };

    Box::into_raw(Box::new(rewriter))
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_write(
    rewriter: *mut HtmlRewriter<'static, ExternOutputSink>,
    chunk: *const c_char,
    chunk_len: size_t,
) -> c_int {
    assert_not_null!(rewriter);
    assert_not_null!(chunk);

    let chunk = bytes_from_raw(chunk, chunk_len);
    let rewriter = unsafe { &mut *rewriter };

    unwrap_or_code! { rewriter.write(chunk) };

    0
}

#[no_mangle]
pub extern "C" fn cool_thing_rewriter_end(
    rewriter: *mut HtmlRewriter<'static, ExternOutputSink>,
) -> c_int {
    assert_not_null!(rewriter);

    let mut rewriter = unsafe { Box::from_raw(rewriter) };

    unwrap_or_code! { rewriter.end() };
    drop(rewriter);

    0
}

#[no_mangle]
pub extern "C" fn cool_thing_doctype_name_get(doctype: *const Doctype<'_>) -> *const Str {
    assert_not_null!(doctype);

    let doctype = unsafe { &*doctype };

    Str::opt_ptr(doctype.name())
}

#[no_mangle]
pub extern "C" fn cool_thing_doctype_public_id_get(doctype: *const Doctype<'_>) -> *const Str {
    assert_not_null!(doctype);

    let doctype = unsafe { &*doctype };

    Str::opt_ptr(doctype.public_id())
}

#[no_mangle]
pub extern "C" fn cool_thing_doctype_system_id_get(doctype: *const Doctype<'_>) -> *const Str {
    assert_not_null!(doctype);

    let doctype = unsafe { &*doctype };

    Str::opt_ptr(doctype.system_id())
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_text_get(comment: *const Comment<'_>) -> *const Str {
    assert_not_null!(comment);

    let comment = unsafe { &*comment };

    Str::ptr(comment.text())
}

#[no_mangle]
pub extern "C" fn cool_thing_comment_text_set(
    comment: *mut Comment<'_>,
    text: *const c_char,
    text_len: size_t,
) -> c_int {
    assert_not_null!(comment);
    assert_not_null!(text);

    let comment = unsafe { &mut *comment };

    let text = unwrap_or_code! { str_from_raw(text, text_len) };

    unwrap_or_code! { comment.set_text(text) };

    0
}

// TODO insertBefore, after, etc.
