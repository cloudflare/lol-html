#![allow(clippy::missing_safety_doc)]

pub use crate::streaming::CStreamingHandler;
use libc::{c_char, c_int, c_void, size_t};
use lol_html::html_content::*;
use lol_html::*;
use std::cell::RefCell;
use std::{ptr, slice, str};
use thiserror::Error;

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
    ($ptr:ident) => {{ unsafe { $ptr.as_ref().expect(concat!(stringify!($var), " is NULL")) } }};
}

macro_rules! to_ref_mut {
    ($ptr:ident) => {{ unsafe { $ptr.as_mut().expect(concat!(stringify!($var), " is NULL")) } }};
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

macro_rules! unwrap_or_ret {
    ($expr:expr, $ret_val:expr) => {
        match $expr {
            Ok(v) => v,
            Err(err) => {
                crate::errors::save_last_error(err.to_string());
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

#[cold]
fn panic_err(payload: Box<dyn std::any::Any + Send>) -> Box<dyn std::error::Error> {
    if let Some(&s) = payload.downcast_ref::<&str>() {
        Box::from(s)
    } else if let Ok(s) = payload.downcast::<String>() {
        Box::from(*s)
    } else {
        Box::from("panic") // never happens
    }
}

fn catch_panic<T, E>(
    callback: impl FnOnce() -> Result<T, E>,
) -> Result<T, Box<dyn std::error::Error>>
where
    Box<dyn std::error::Error>: From<E>,
{
    Ok(std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)).map_err(panic_err)??)
}

macro_rules! impl_content_mutation_handlers {
    ($name:ident: $typ:ty [ $($(#[$meta:meta])* $(@$kind:ident)? $fn_name:ident => $method:ident),+$(,)? ]) => {
        $(
            // stable Rust can't concatenate idents, so fn_name must be written out manually,
            // but it is possible to compare concatenated strings.
            #[cfg(debug_assertions)]
            const _: () = {
                let expected_fn_name_prefix = concat!("lol_html_", stringify!($name), "_").as_bytes();
                let fn_name = stringify!($fn_name).as_bytes();
                // removed vs is_removed prevents exact comparison
                assert!(fn_name.len() >= expected_fn_name_prefix.len() + (stringify!($method).len()), stringify!($fn_name));
                let mut i = 0;
                while i < expected_fn_name_prefix.len() {
                    assert!(expected_fn_name_prefix[i] == fn_name[i], stringify!($fn_name));
                    i += 1;
                }
            };
            impl_content_mutation_handlers! { IMPL $($kind)? $name: $typ, $(#[$meta])* $fn_name => $method }
        )+
    };
    (IMPL $name:ident: $typ:ty, $fn_name:ident => source_location_bytes) => {
        /// Returns [`SourceLocationBytes`].
        ///
        #[doc = concat!(" `", stringify!($name), "` must be valid and non-`NULL`.")]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $fn_name($name: *mut $typ) -> SourceLocationBytes {
            let loc = to_ref_mut!($name).source_location().bytes();
            SourceLocationBytes {
                start: loc.start,
                end: loc.end,
            }
        }
    };
    (IMPL $name:ident: $typ:ty, $(#[$meta:meta])* $fn_name:ident => $method:ident) => {
        $(#[$meta])*
        /// The `content` must be a valid UTF-8 string. It's copied immediately.
        /// If `is_html` is `true`, then the `content` will be written without HTML-escaping.
        ///
        #[doc = concat!(" `", stringify!($name), "` must be valid and non-`NULL`.")]
        /// If `content` is `NULL`, an error will be reported.
        ///
        /// Returns 0 on success.
        ///
        #[doc = concat!(" Calls [`", stringify!($typ), "::", stringify!($method), "`].")]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $fn_name(
            $name: *mut $typ,
            content: *const c_char,
            content_len: size_t,
            is_html: bool,
        ) -> c_int {
            content_insertion_fn_body! { $name.$method(content, content_len, is_html) }
        }
    };
    (IMPL STREAM $name:ident: $typ:ty, $(#[$meta:meta])* $fn_name:ident => $method:ident) => {
        $(#[$meta])*
        /// The [`CStreamingHandler`] contains callbacks that will be called
        /// when the content needs to be written.
        ///
        /// `streaming_writer` is copied immediately, and doesn't have a stable address.
        /// `streaming_writer` may be used from another thread (`Send`), but it's only going
        /// to be used by one thread at a time (`!Sync`).
        ///
        #[doc = concat!(" `", stringify!($name), "` must be valid and non-`NULL`.")]
        /// If `streaming_writer` is `NULL`, an error will be reported.
        ///
        /// Returns 0 on success.
        ///
        #[doc = concat!(" Calls [`", stringify!($typ), "::", stringify!($method), "`].")]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $fn_name(
            $name: *mut $typ,
            streaming_writer: *mut CStreamingHandler,
        ) -> c_int {
            content_insertion_fn_body! { $name.$method(streaming_writer) }
        }
    };
    (IMPL VOID $name:ident: $typ:ty, $(#[$meta:meta])* $fn_name:ident => $method:ident) => {
        $(#[$meta])*
        #[doc = concat!(" `", stringify!($name), "` must be valid and non-`NULL`.")]
        ///
        #[doc = concat!(" Calls [`", stringify!($typ), "::", stringify!($method), "`].")]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $fn_name(
            $name: *mut $typ,
        ) {
            to_ref_mut!($name).$method();
        }
    };
    (IMPL BOOL $name:ident: $typ:ty, $(#[$meta:meta])* $fn_name:ident => $method:ident) => {
        $(#[$meta])*
        #[doc = concat!(" `", stringify!($name), "` must be valid and non-`NULL`.")]
        /// Returns `_Bool`.
        ///
        #[doc = concat!(" Calls [`", stringify!($typ), "::", stringify!($method), "`].")]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $fn_name(
            $name: *mut $typ,
        ) -> bool {
            to_ref_mut!($name).$method()
        }
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
    ($target:ident.$method:ident($handler:expr)) => {{
        let handler_ptr: *mut CStreamingHandler = $handler;
        if unsafe { handler_ptr.as_ref() }.is_none_or(|handler| !handler.reserved.is_null()) {
            // we can't even safely call drop callback on this
            return -1;
        }
        // Taking ownership of the CStreamingHandler
        let handler: Box<CStreamingHandler> = Box::new(unsafe { handler_ptr.read() });
        if handler.write_all_callback.is_none() {
            return -1;
        }
        if let Some(target) = unsafe { $target.as_mut() } {
            target.$method(handler);
            0
        } else {
            -1
        }
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

pub mod comment;
pub mod doctype;
pub mod document_end;
pub mod element;
pub mod errors;
pub mod rewriter;
pub mod rewriter_builder;
pub mod selector;
pub mod streaming;
pub mod string;
pub mod text_chunk;

pub use self::string::Str;

/// `size_t` byte offsets from the start of the input document
#[repr(C)]
pub struct SourceLocationBytes {
    pub start: usize,
    pub end: usize,
}

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
