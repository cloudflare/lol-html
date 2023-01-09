#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate encoding_rs;
extern crate lol_html;
extern crate rand;

extern crate libc;

use libc::{c_char, c_void, size_t};
use rand::Rng;
use std::ffi::{CStr, CString};

use encoding_rs::*;
use lol_html::html_content::ContentType;
use lol_html::{
    comments, doc_comments, doc_text, element, text, HtmlRewriter, MemorySettings, Settings,
};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

static ASCII_COMPATIBLE_ENCODINGS: [&Encoding; 36] = [
    BIG5,
    EUC_JP,
    EUC_KR,
    GB18030,
    GBK,
    IBM866,
    ISO_8859_2,
    ISO_8859_3,
    ISO_8859_4,
    ISO_8859_5,
    ISO_8859_6,
    ISO_8859_7,
    ISO_8859_8,
    ISO_8859_8_I,
    ISO_8859_10,
    ISO_8859_13,
    ISO_8859_14,
    ISO_8859_15,
    ISO_8859_16,
    KOI8_R,
    KOI8_U,
    MACINTOSH,
    SHIFT_JIS,
    UTF_8,
    WINDOWS_874,
    WINDOWS_1250,
    WINDOWS_1251,
    WINDOWS_1252,
    WINDOWS_1253,
    WINDOWS_1254,
    WINDOWS_1255,
    WINDOWS_1256,
    WINDOWS_1257,
    WINDOWS_1258,
    X_MAC_CYRILLIC,
    X_USER_DEFINED,
];

static SUPPORTED_SELECTORS: [&str; 16] = [
    "*",
    "p",
    "p:not(.firstline)",
    "p.warning",
    "p#myid",
    "p[foo]",
    "p[foo=\"bar\"]",
    "p[foo=\"bar\" i]",
    "p[foo=\"bar\" s]",
    "p[foo~=\"bar\"]",
    "p[foo^=\"bar\"]",
    "p[foo$=\"bar\"]",
    "p[foo*=\"bar\"]",
    "p[foo|=\"bar\"]",
    "p a",
    "p > a",
];

extern "C" fn empty_handler(_foo: *const c_char, _size: size_t, _boo: *mut c_void) -> () {}

pub fn run_rewriter(data: &[u8]) -> () {
    // fuzzing with randomly picked selector and encoding
    // works much faster (50 times) that iterating over all
    // selectors/encoding per single run. It's recommended
    // to make iterations as fast as possible per fuzzing docs.
    run_rewriter_iter(data, get_random_selector(), get_random_encoding());
}

pub fn run_c_api_rewriter(data: &[u8]) -> () {
    run_c_api_rewriter_iter(data, get_random_encoding().name());
}

fn get_random_encoding() -> &'static Encoding {
    let random_encoding_index = rand::thread_rng().gen_range(0..ASCII_COMPATIBLE_ENCODINGS.len());
    return ASCII_COMPATIBLE_ENCODINGS[random_encoding_index];
}

fn get_random_selector() -> &'static str {
    let random_selector_index = rand::thread_rng().gen_range(0..SUPPORTED_SELECTORS.len());
    return SUPPORTED_SELECTORS[random_selector_index];
}

fn run_rewriter_iter(data: &[u8], selector: &str, encoding: &'static Encoding) -> () {
    let mut rewriter = HtmlRewriter::new(
        Settings {
            enable_esi_tags: true,
            element_content_handlers: vec![
                element!(selector, |el| {
                    el.before(
                        &format!("<!--[ELEMENT('{}')]-->", selector),
                        ContentType::Html,
                    );
                    el.after(
                        &format!("<!--[/ELEMENT('{}')]-->", selector),
                        ContentType::Html,
                    );
                    el.set_inner_content(
                        &format!("<!--Replaced ({}) -->", selector),
                        ContentType::Html,
                    );

                    Ok(())
                }),
                comments!(selector, |c| {
                    c.before(
                        &format!("<!--[COMMENT('{}')]-->", selector),
                        ContentType::Html,
                    );
                    c.after(
                        &format!("<!--[/COMMENT('{}')]-->", selector),
                        ContentType::Html,
                    );

                    Ok(())
                }),
                text!(selector, |t| {
                    t.before(&format!("<!--[TEXT('{}')]-->", selector), ContentType::Html);

                    if t.last_in_text_node() {
                        t.after(
                            &format!("<!--[/TEXT('{}')]-->", selector),
                            ContentType::Html,
                        );
                    }

                    Ok(())
                }),
                element!(selector, |el| {
                    el.replace("hey & ya", ContentType::Html);

                    Ok(())
                }),
                element!(selector, |el| {
                    el.remove();

                    Ok(())
                }),
                element!(selector, |el| {
                    el.remove_and_keep_content();

                    Ok(())
                }),
            ],
            document_content_handlers: vec![
                doc_comments!(|c| {
                    c.set_text(&"123456").unwrap();

                    Ok(())
                }),
                doc_text!(|t| {
                    if t.last_in_text_node() {
                        t.after("BAZ", ContentType::Text);
                    }

                    Ok(())
                }),
            ],
            encoding: encoding.try_into().unwrap(),
            memory_settings: MemorySettings::default(),
            strict: false,
        },
        |_: &[u8]| {},
    );

    rewriter.write(data).unwrap();
    rewriter.end().unwrap();
}

fn run_c_api_rewriter_iter(data: &[u8], encoding: &str) -> () {
    let c_encoding = CString::new(encoding).expect("CString::new failed.");

    unsafe {
        let builder = lol_html_rewriter_builder_new();
        let mut output_data = {};
        let output_data_ptr: *mut c_void = &mut output_data as *mut _ as *mut c_void;

        let rewriter = lol_html_rewriter_build(
            builder,
            c_encoding.as_ptr(),
            encoding.len(),
            lol_html_memory_settings_t {
                preallocated_parsing_buffer_size: 0,
                max_allowed_memory_usage: std::usize::MAX,
            },
            Some(empty_handler),
            output_data_ptr,
            false,
        );

        let cstr = CStr::from_bytes_with_nul_unchecked(data);

        lol_html_rewriter_write(rewriter, cstr.as_ptr(), data.len());
        lol_html_rewriter_builder_free(builder);
        lol_html_rewriter_free(rewriter);
    }
}
