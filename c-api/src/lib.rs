use libc::{c_char, c_int, c_void, size_t};
use lol_html::html_content::*;
use lol_html::*;
use std::cell::RefCell;
use std::{ptr, slice, str};
use thiserror::Error;

#[inline]
fn to_ptr<T>(val: T) -> *const T {
    Box::into_raw(Box::new(val))
}

#[inline]
fn to_ptr_mut<T>(val: T) -> *mut T {
    Box::into_raw(Box::new(val))
}

// NOTE: abort the thread if we receive NULL where unexpected
macro_rules! assert_not_null {
    ($var:ident) => {
        assert!(!$var.is_null(), "{} is NULL", stringify!($var));
    };
}

// NOTE: all these utilities are macros so we can propagate the variable
// name to the null pointer assertion.
macro_rules! to_ref {
    ($ptr:ident) => {{
        assert_not_null!($ptr);
        unsafe { &*$ptr }
    }};
}

macro_rules! to_ref_mut {
    ($ptr:ident) => {{
        assert_not_null!($ptr);
        unsafe { &mut *$ptr }
    }};
}

macro_rules! to_box {
    ($ptr:ident) => {{
        assert_not_null!($ptr);
        unsafe { Box::from_raw($ptr) }
    }};
}

macro_rules! to_bytes {
    ($data:ident, $len:ident) => {{
        assert_not_null!($data);
        unsafe { slice::from_raw_parts($data as *const u8, $len) }
    }};
}

macro_rules! to_str {
    ($data:ident, $len:ident) => {
        str::from_utf8(to_bytes!($data, $len)).into()
    };
}

macro_rules! static_c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const c_char
    };
}

macro_rules! unwrap_or_ret {
    ($expr:expr, $ret_val:expr) => {
        match $expr {
            Ok(v) => v,
            Err(err) => {
                crate::errors::LAST_ERROR.with(|e| *e.borrow_mut() = Some(err.into()));
                return $ret_val;
            }
        }
    };
}

macro_rules! unwrap_or_ret_err_code {
    ($expr:expr) => {
        unwrap_or_ret!($expr, -1)
    };
}

macro_rules! unwrap_or_ret_null {
    ($expr:expr) => {
        unwrap_or_ret!($expr, ptr::null_mut())
    };
}

macro_rules! content_insertion_fn_body {
    ($target:ident.$method:ident($content:ident, $content_len:ident, $is_html:ident)) => {{
        let target = to_ref_mut!($target);
        let content = unwrap_or_ret_err_code! { to_str!($content, $content_len) };

        target.$method(
            content,
            if $is_html {
                ContentType::Html
            } else {
                ContentType::Text
            },
        );

        0
    }};
}

macro_rules! get_user_data {
    ($unit:ident) => {
        to_ref!($unit)
            .user_data()
            .downcast_ref::<*mut c_void>()
            .map(|d| *d)
            .unwrap_or(ptr::null_mut())
    };
}

mod comment;
mod doctype;
mod document_end;
mod element;
mod errors;
mod rewriter;
mod rewriter_builder;
mod selector;
mod string;
mod text_chunk;

pub use self::string::Str;

// NOTE: prevent dead code from complaining about enum
// never being constructed in the Rust code.
pub use self::rewriter_builder::RewriterDirective;

/// An error that occurs if incorrect [`encoding`] label was provided in [`Settings`].
///
/// [`encoding`]: ../struct.Settings.html#structfield.encoding
/// [`Settings`]: ../struct.Settings.html
#[derive(Error, Debug, PartialEq, Copy, Clone)]
pub enum EncodingError {
    /// The provided value doesn't match any of the [labels specified in the standard].
    ///
    /// [labels specified in the standard]: https://encoding.spec.whatwg.org/#names-and-labels
    #[error("Unknown character encoding has been provided.")]
    UnknownEncoding,

    /// The provided label is for one of the non-ASCII-compatible encodings (`UTF-16LE`, `UTF-16BE`,
    /// `ISO-2022-JP` and `replacement`). These encodings are not supported.
    #[error("Expected ASCII-compatible encoding.")]
    NonAsciiCompatibleEncoding,
}
