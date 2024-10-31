use super::ContentType;
use crate::html::escape_body_text;
use encoding_rs::{CoderResult, Encoder, Encoding, UTF_8};

/// Used to write chunks of text or markup in streaming mutation handlers.
///
/// Argument to [`StreamingHandler::write_all()`](crate::html_content::StreamingHandler::write_all).
pub struct StreamingHandlerSink<'output_handler> {
    non_utf8_encoder: Option<TextEncoder>,

    /// ```compile_fail
    /// use lol_html::html_content::StreamingHandlerSink;
    /// struct IsSend<T: Send>(T);
    /// let x: IsSend<StreamingHandlerSink<'static>>;
    /// ```
    ///
    /// ```compile_fail
    /// use lol_html::html_content::StreamingHandlerSink;
    /// struct IsSync<T: Sync>(T);
    /// let x: IsSync<StreamingHandlerSink<'static>>;
    /// ```
    output_handler: &'output_handler mut dyn FnMut(&[u8]),
}

impl<'output_handler> StreamingHandlerSink<'output_handler> {
    #[inline(always)]
    pub(crate) fn new(
        encoding: &'static Encoding,
        output_handler: &'output_handler mut dyn FnMut(&[u8]),
    ) -> Self {
        Self {
            non_utf8_encoder: (encoding != UTF_8).then(|| TextEncoder::new(encoding)),
            output_handler,
        }
    }

    /// Writes the given UTF-8 string to the output, converting the encoding and [escaping](ContentType) if necessary.
    ///
    /// It may be called multiple times. The strings will be concatenated together.
    #[inline]
    pub fn write_str(&mut self, content: &str, content_type: ContentType) {
        match content_type {
            ContentType::Html => self.write_html(content),
            ContentType::Text => self.write_body_text(content),
        }
    }

    pub(crate) fn write_html(&mut self, html: &str) {
        if let Some(encoder) = &mut self.non_utf8_encoder {
            encoder.encode(html, self.output_handler);
        } else if !html.is_empty() {
            (self.output_handler)(html.as_bytes());
        }
    }

    /// For text content, not attributes
    pub(crate) fn write_body_text(&mut self, plaintext: &str) {
        if let Some(encoder) = &mut self.non_utf8_encoder {
            escape_body_text(plaintext, &mut |chunk| {
                debug_assert!(!chunk.is_empty());
                encoder.encode(chunk, self.output_handler);
            });
        } else {
            escape_body_text(plaintext, &mut |chunk| {
                debug_assert!(!chunk.is_empty());
                (self.output_handler)(chunk.as_bytes());
            });
        }
    }

    #[inline]
    pub(crate) fn output_handler(&mut self) -> &mut dyn FnMut(&[u8]) {
        &mut self.output_handler
    }
}

enum Buffer {
    Heap(Vec<u8>),
    Stack([u8; 63]), // leave a byte for the tag
}

struct TextEncoder {
    encoder: Encoder,
    buffer: Buffer,
}

impl TextEncoder {
    #[inline]
    pub fn new(encoding: &'static Encoding) -> Self {
        debug_assert!(encoding != UTF_8);
        debug_assert!(encoding.is_ascii_compatible());
        Self {
            encoder: encoding.new_encoder(),
            buffer: Buffer::Stack([0; 63]),
        }
    }

    /// This is more efficient than `Bytes::from_str`, because it can output non-UTF-8/non-ASCII encodings
    /// without heap allocations.
    /// It also avoids methods that have UB: https://github.com/hsivonen/encoding_rs/issues/79
    #[inline(never)]
    fn encode(&mut self, mut content: &str, output_handler: &mut dyn FnMut(&[u8])) {
        loop {
            debug_assert!(!self.encoder.has_pending_state()); // ASCII-compatible encodings are not supposed to have it
            let ascii_len = Encoding::ascii_valid_up_to(content.as_bytes());
            if let Some((ascii, remainder)) = content.split_at_checked(ascii_len) {
                if !ascii.is_empty() {
                    (output_handler)(ascii.as_bytes());
                }
                if remainder.is_empty() {
                    return;
                }
                content = remainder;
            }

            let buffer = match &mut self.buffer {
                Buffer::Heap(buf) => buf.as_mut_slice(),
                // Long non-ASCII content could take lots of roundtrips through the encoder
                buf if content.len() >= 1 << 20 => {
                    *buf = Buffer::Heap(vec![0; 4096]);
                    match buf {
                        Buffer::Heap(buf) => buf.as_mut(),
                        _ => unreachable!(),
                    }
                }
                Buffer::Stack(buf) => buf.as_mut_slice(),
            };

            let (result, read, written, _) = self.encoder.encode_from_utf8(content, buffer, false);
            if written > 0 && written <= buffer.len() {
                (output_handler)(&buffer[..written]);
            }
            if read >= content.len() {
                return;
            }
            content = &content[read..];
            match result {
                CoderResult::InputEmpty => {
                    debug_assert!(content.is_empty());
                    return;
                }
                CoderResult::OutputFull => {
                    match &mut self.buffer {
                        Buffer::Heap(buf) if buf.len() >= 1024 => {
                            if written == 0 {
                                panic!("encoding_rs infinite loop"); // encoding_rs only needs a dozen bytes
                            }
                        }
                        buf => *buf = Buffer::Heap(vec![0; 1024]),
                    }
                }
            }
        }
    }
}

#[test]
fn long_text() {
    let mut written = 0;
    let mut expected = 0;
    let mut handler = |ch: &[u8]| {
        assert!(
            ch.iter().all(|&c| {
                written += 1;
                c == if 0 != written & 1 {
                    177
                } else {
                    b'0' + ((written / 2 - 1) % 10) as u8
                }
            }),
            "@{written} {ch:?}"
        );
    };
    let mut t = StreamingHandlerSink::new(encoding_rs::ISO_8859_2, &mut handler);

    let mut s = "ą0ą1ą2ą3ą4ą5ą6ą7ą8ą9".repeat(128);
    while s.len() <= 1 << 17 {
        s.push_str(&s.clone());
        expected += s.chars().count();
        t.write_str(&s, ContentType::Text);
    }
    assert_eq!(expected, written);
}
