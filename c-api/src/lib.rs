use cool_thing::*;
use failure::Error;
use libc::{c_char, c_int, size_t};
use std::cell::RefCell;
use std::ptr;
use std::slice;
use std::str;

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
        str::from_utf8(to_bytes!($data, $len)).map_err(Error::from)
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

mod comment;
mod doctype;
mod element;
mod errors;
mod rewriter;
mod rewriter_builder;
mod string;
mod text_chunk;

pub use self::string::Str;
