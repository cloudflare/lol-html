#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// make it link
use lolhtml as _;

use libc::{c_char, c_void, size_t};
use rand::Rng;
use std::ffi::{CStr, CString};

use encoding_rs::*;
use lol_html::html_content::ContentType;
use lol_html::{comments, doc_comments, doc_text, element, streaming, text};
use lol_html::{HtmlRewriter, MemorySettings, Settings};

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

static MARKUP: &[&str] = &[
    "<",
    "/>",
    "</",
    "\"",
    "\'",
    " = ",
    "<p>",
    "</p>",
    "<br>",
    "<svg>",
    "</style>",
    "</script>",
    "<!-",
    "<!--",
    "->",
    "--!>",
    "-->",
    " = \"",
    "&&&&",
    "&amp",
    "&copy",
    "&lt;",
    "&#",
    "<math>",
    "<mi>",
    "<link>",
    "<body",
    "<head",
    ">>>",
    "<p id=foo>",
    "<p foo=x>",
    "<p foo=bar>",
    "<p",
    "foo=BAR",
    "<a",
    "id=myid",
    "class='warning ",
    "class=\"",
    "       ",
    "\n\r\n\r\r\t",
    "&quo",
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

extern "C" fn empty_handler(_foo: *const c_char, _size: size_t, _boo: *mut c_void) {}

fn get_byte(data: &mut &[u8]) -> u8 {
    let Some((first, rest)) = (*data).split_at_checked(1) else {
        return 1;
    };
    *data = rest;
    first[0]
}

fn get_bytes<'b>(data: &mut &'b [u8], min_len: usize) -> &'b [u8] {
    let len = min_len + get_byte(data) as usize;
    let (slice, rest) = data.split_at(data.len().min(len));
    *data = rest;
    slice
}

fn get_string<'b>(data: &mut &'b [u8], min_len: usize) -> String {
    let len = min_len + get_byte(data) as usize;
    let (rest, slice) = data.split_at(data.len() - data.len().min(len));
    *data = rest;
    slice
        .chunks(2)
        .flat_map(|ch| {
            let c1 = ch[0];
            let c2 = ch.get(1).copied().unwrap_or(b'>');
            if c1 > 0 && c1 < 128 {
                [c1 as char, c2 as char].into_iter().take(2)
            } else {
                [
                    char::from_u32(c1 as u32 + 333 * c2 as u32).unwrap_or('<'),
                    ' ',
                ]
                .into_iter()
                .take(1)
            }
        })
        .collect()
}

pub fn run_rewriter(mut data: &[u8]) {
    let settings = get_byte(&mut data);

    let encoding =
        ASCII_COMPATIBLE_ENCODINGS[(settings as usize / 5) % ASCII_COMPATIBLE_ENCODINGS.len()];

    let mut rewriter = HtmlRewriter::new_send(
        Settings {
            enable_esi_tags: settings & 1 == 0,
            element_content_handlers: (0..(get_byte(&mut data) % 7))
                .map(|i| {
                    let n = get_byte(&mut data) as usize;
                    let m = get_byte(&mut data) as usize;
                    let num_selectors = 1 + ((n / 13) % 5).min(i as usize);
                    let selector = (0..num_selectors)
                        .map(|n| SUPPORTED_SELECTORS[n % SUPPORTED_SELECTORS.len()])
                        .collect::<Vec<_>>()
                        .join([" > ", ",", " "][m % 3]);
                    match n % 5 {
                        0 => text!(selector, move |t| {
                            for _ in 0..1+i {
                                if m & 32 == 0 {
                                    let s = get_string(&mut data, 1);
                                    t.streaming_replace(streaming!(move |sink| {
                                        for chunk in s.as_bytes().chunks(1+m) {
                                            sink.write_utf8_chunk(chunk, ContentType::Html)?;
                                        }
                                        Ok(())
                                    }));
                                }
                                if m & 16 == 0 {
                                    t.replace(&get_string(&mut data, 1), ContentType::Html);
                                }
                                if m & 8 == 0 {
                                    let s = get_string(&mut data, 1);
                                    t.streaming_before(streaming!(move |sink| {
                                        for chunk in s.as_bytes().chunks(1+m) {
                                            sink.write_utf8_chunk(chunk, ContentType::Html)?;
                                        }
                                        Ok(())
                                    }));
                                }
                                if m & 4 == 0 {
                                    t.before(&get_string(&mut data, 1), ContentType::Html);
                                }
                                if m & 2 == 0 {
                                    t.after(&get_string(&mut data, 10), ContentType::Html);
                                }
                            }
                            Ok(())
                        }),
                        1 => comments!(selector, move |c| {
                            if m & 32 == 0 {
                                let s = get_string(&mut data, 1);
                                c.streaming_replace(streaming!(move |sink| {
                                    for chunk in s.as_bytes().chunks(1+m) {
                                        sink.write_utf8_chunk(chunk, ContentType::Html)?;
                                    }
                                    Ok(())
                                }));
                            }
                            if m & 16 == 0 {
                                c.replace(&get_string(&mut data, 1), ContentType::Html);
                            }
                            if m & 8 == 0 {
                                let s = get_string(&mut data, 1);
                                c.streaming_before(streaming!(move |sink| {
                                    for chunk in s.as_bytes().chunks(1+m) {
                                        sink.write_utf8_chunk(chunk, ContentType::Html)?;
                                    }
                                    Ok(())
                                }));
                            }
                            if m & 4 == 0 {
                                c.before(&get_string(&mut data, 10), ContentType::Html);
                            }
                            if m & 2 == 0 {
                                c.after(&get_string(&mut data, 1), ContentType::Html);
                            }
                            Ok(())
                        }),
                        _ => element!(selector, move |c| {
                            for &b in get_bytes(&mut data, 1).iter().take(4) {
                                let m = m ^ b as usize;
                                if m & 128 == 0 {
                                    let s = get_string(&mut data, 1);
                                    c.streaming_append(streaming!(move |sink| {
                                        for chunk in s.as_bytes().chunks(1+m) {
                                            sink.write_utf8_chunk(chunk, ContentType::Html)?;
                                        }
                                        Ok(())
                                    }));
                                }
                                if m & 64 == 0 {
                                    c.prepend(&get_string(&mut data, 1), ContentType::Html);
                                }
                                if m & 32 == 0 {
                                    let s = get_string(&mut data, 1);
                                    c.streaming_replace(streaming!(move |sink| {
                                        for chunk in s.as_bytes().chunks(1+m) {
                                            sink.write_utf8_chunk(chunk, ContentType::Html)?;
                                        }
                                        Ok(())
                                    }));
                                }
                                if m & 16 == 0 {
                                    c.replace(&get_string(&mut data, 1), ContentType::Html);
                                }
                                if m & 8 == 0 {
                                    let s = get_string(&mut data, 1);
                                    c.streaming_before(streaming!(move |sink| {
                                        for chunk in s.as_bytes().chunks(1+m) {
                                            sink.write_utf8_chunk(chunk, ContentType::Html)?;
                                        }
                                        Ok(())
                                    }));
                                }
                                if m & 4 == 0 {
                                    c.before(&get_string(&mut data, 10), ContentType::Html);
                                }
                                if m & 2 == 0 {
                                    c.after(&get_string(&mut data, 1), ContentType::Html);
                                }
                            }
                            Ok(())
                        }),
                    }
                })
                .collect(),
            document_content_handlers: (0..get_byte(&mut data) % 5).map(|i| {
                    let s = get_string(&mut data, 1);
                    if i & 1 == 0 {
                        doc_comments!(move |c| {
                            let _ = c.set_text(&s); // lots of reasons for random text to fail
                            Ok(())
                        })
                    } else {
                        doc_text!(move |t| {
                            if t.last_in_text_node() {
                                t.after(&s, ContentType::Text);
                            }
                            Ok(())
                        })
                    }
                }).collect(),
            encoding: encoding.try_into().unwrap(),
            memory_settings: MemorySettings::new(),
            strict: settings & 2 == 0,
            adjust_charset_on_meta_tag: settings & 4 == 0,
        },
        |_: &[u8]| {},
    );

    for chunk in data.chunks(settings as usize / 7 + 13) {
        rewriter.write(MARKUP[chunk[0] as usize % MARKUP.len()].as_bytes()).unwrap();
        rewriter.write(chunk).unwrap();
    }

    rewriter.end().unwrap();
}

pub fn run_random_rewriter(data: &[u8]) {
    // fuzzing with randomly picked selector and encoding
    // works much faster (50 times) that iterating over all
    // selectors/encoding per single run. It's recommended
    // to make iterations as fast as possible per fuzzing docs.
    run_rewriter_iter(data, get_random_selector(), get_random_encoding());
}

pub fn run_c_api_rewriter(data: &[u8]) {
    run_c_api_rewriter_iter(data, get_random_encoding().name());
}

fn get_random_encoding() -> &'static Encoding {
    let random_encoding_index = rand::thread_rng().gen_range(0..ASCII_COMPATIBLE_ENCODINGS.len());
    ASCII_COMPATIBLE_ENCODINGS[random_encoding_index]
}

fn get_random_selector() -> &'static str {
    let random_selector_index = rand::thread_rng().gen_range(0..SUPPORTED_SELECTORS.len());
    SUPPORTED_SELECTORS[random_selector_index]
}

fn run_rewriter_iter(data: &[u8], selector: &str, encoding: &'static Encoding) {
    let mut rewriter: HtmlRewriter<_> = HtmlRewriter::new(
        Settings {
            enable_esi_tags: true,
            element_content_handlers: vec![
                element!(selector, |el| {
                    el.before(
                        &format!("<!--[ELEMENT('{selector}')]-->"),
                        ContentType::Html,
                    );
                    el.after(
                        &format!("<!--[/ELEMENT('{selector}')]-->"),
                        ContentType::Html,
                    );

                    let replaced = format!("<!--Replaced ({selector}) -->");
                    el.streaming_set_inner_content(streaming!(move |sink| {
                        sink.write_str(&replaced, ContentType::Html);
                        Ok(())
                    }));

                    Ok(())
                }),
                comments!(selector, |c| {
                    c.before(
                        &format!("<!--[COMMENT('{selector}')]-->"),
                        ContentType::Html,
                    );
                    c.after(
                        &format!("<!--[/COMMENT('{selector}')]-->"),
                        ContentType::Html,
                    );

                    Ok(())
                }),
                text!(selector, |t| {
                    t.before(&format!("<!--[TEXT('{selector}')]-->"), ContentType::Html);

                    if t.last_in_text_node() {
                        t.after(&format!("<!--[/TEXT('{selector}')]-->"), ContentType::Html);
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
                    c.set_text("123456").unwrap();

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
            memory_settings: MemorySettings::new(),
            strict: false,
            adjust_charset_on_meta_tag: false,
        },
        |_: &[u8]| {},
    );

    rewriter.write(data).unwrap();
    rewriter.end().unwrap();
}

fn run_c_api_rewriter_iter(data: &[u8], encoding: &str) {
    let c_encoding = CString::new(encoding).expect("CString::new failed.");

    unsafe {
        let builder = lol_html_rewriter_builder_new();
        let mut output_data = ();
        let output_data_ptr: *mut c_void = std::ptr::from_mut(&mut output_data).cast::<c_void>();

        let rewriter = lol_html_rewriter_build(
            builder,
            c_encoding.as_ptr(),
            encoding.len(),
            lol_html_memory_settings_t {
                preallocated_parsing_buffer_size: 0,
                max_allowed_memory_usage: usize::MAX,
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
