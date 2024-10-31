use super::ContentType;
use crate::html::escape_body_text;
use encoding_rs::{CoderResult, Encoder, Encoding, UTF_8};
use thiserror::Error;

/// Input contained non-UTF-8 byte sequence
///
/// [`StreamingHandlerSink::write_utf8_chunk`] will not fail on an incomplete UTF-8 sequence at the end of the chunk,
/// but it will report errors if incomplete UTF-8 sequences are within the chunk, or the next call starts with
/// bytes that don't match the previous call's trailing bytes.
#[derive(Error, Debug, Eq, PartialEq, Copy, Clone)]
#[error("Invalid UTF-8")]
pub struct Utf8Error;

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

    /// Half-written UTF-8
    incomplete_bytes: [u8; 4],
    incomplete_bytes_len: u8,
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
            incomplete_bytes: [0; 4],
            incomplete_bytes_len: 0,
        }
    }

    /// Writes the given UTF-8 string to the output, converting the encoding and [escaping](ContentType) if necessary.
    ///
    /// It may be called multiple times. The strings will be concatenated together.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn write_str(&mut self, content: &str, content_type: ContentType) {
        match content_type {
            ContentType::Html => self.write_html(content),
            ContentType::Text => self.write_body_text(content),
        }
    }

    /// Writes as much of the given UTF-8 fragment as possible, converting the encoding and [escaping](ContentType) if necessary.
    ///
    /// The `content` doesn't need to be a complete UTF-8 string, as long as consecutive calls to `write_utf8_bytes` create a valid UTF-8 string.
    /// Any incomplete UTF-8 sequence at the end of the content is buffered and flushed as soon as it's completed.
    ///
    /// Other methods like `write_str_chunk` should not be called after a `write_utf8_bytes` call with an incomplete UTF-8 sequence.
    #[inline]
    pub fn write_utf8_chunk(
        &mut self,
        content: &[u8],
        content_type: ContentType,
    ) -> Result<(), Utf8Error> {
        let content = self.make_complete_utf8(content).ok_or(Utf8Error)?;
        if content.is_empty() {
            return Ok(());
        }

        match content_type {
            ContentType::Html => self.write_html_inner(content),
            ContentType::Text => self.write_body_text(content),
        };
        Ok(())
    }

    fn make_complete_utf8<'c>(&mut self, mut content: &'c [u8]) -> Option<&'c str> {
        // Finish previous incomplete sequence if possible
        while let Some((&first, rest)) = content.split_first() {
            if is_utf8_char_boundary(first) {
                break;
            }
            let pos = self.incomplete_bytes_len as usize;
            *self.incomplete_bytes.get_mut(pos)? = first;
            self.incomplete_bytes_len += 1;
            content = rest;
            if content.is_empty() {
                if self.incomplete_bytes_len > 1 {
                    // this could have been the end of the string with the last char completed now
                    let _ = self.flush_incomplete_utf8();
                }
                return Some("");
            }
        }
        // Found a new char boundary, so the buffer must contain a valid UTF-8 sequence
        if self.incomplete_bytes_len > 0 {
            self.flush_incomplete_utf8()?;
        }

        match std::str::from_utf8(content) {
            Ok(content) => Some(content),
            Err(err) => {
                // error_len means invalid bytes, not just incomplete
                if err.error_len().is_none() {
                    let (valid, invalid) = content.split_at_checked(err.valid_up_to())?;
                    if invalid.len() <= 3 {
                        debug_assert_eq!(0, self.incomplete_bytes_len);
                        self.incomplete_bytes[..invalid.len()].copy_from_slice(invalid);
                        self.incomplete_bytes_len = invalid.len() as _;
                        // valid_up_to promises it is valid
                        debug_assert!(std::str::from_utf8(valid).is_ok());
                        return Some(unsafe { std::str::from_utf8_unchecked(valid) });
                    }
                }
                None
            }
        }
    }

    #[inline]
    fn flush_incomplete_utf8(&mut self) -> Option<()> {
        debug_assert!(self.incomplete_bytes_len > 0);
        let tmp = self.incomplete_bytes;
        let completed_char = tmp.get(..self.incomplete_bytes_len as usize)?;
        let completed_char = std::str::from_utf8(completed_char).ok()?;
        self.incomplete_bytes_len = 0;
        self.write_html_inner(completed_char); // it's non-ASCII, so it can't contain chars to escape
        Some(())
    }

    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn discard_incomplete_utf8(&mut self) {
        debug_assert_eq!(
            0, self.incomplete_bytes_len,
            "Previous write had unfinished UTF-8 sequence"
        );

        if self.incomplete_bytes_len > 0 {
            // It's not possible for a valid UTF-8 string to complete a previously-incomplete sequence
            self.incomplete_bytes_len = 0;
            self.write_html_inner("\u{fffd}");
        }
    }

    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn write_html(&mut self, html: &str) {
        self.discard_incomplete_utf8();
        self.write_html_inner(html);
    }

    pub(crate) fn write_html_inner(&mut self, html: &str) {
        if let Some(encoder) = &mut self.non_utf8_encoder {
            encoder.encode(html, self.output_handler);
        } else if !html.is_empty() {
            (self.output_handler)(html.as_bytes());
        }
    }

    /// For text content, not attributes
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn write_body_text(&mut self, plaintext: &str) {
        self.discard_incomplete_utf8();
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
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn output_handler(&mut self) -> &mut dyn FnMut(&[u8]) {
        self.discard_incomplete_utf8();
        &mut self.output_handler
    }
}

pub(crate) const fn is_utf8_char_boundary(b: u8) -> bool {
    b < 128 || b >= 192
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
            content = &content[..read];
            match result {
                CoderResult::InputEmpty => return,
                CoderResult::OutputFull => {
                    match &mut self.buffer {
                        Buffer::Heap(buf) if buf.len() >= 1024 => {
                            panic!("encoding_rs infinite loop"); // encoding_rs only needs a dozen bytes
                        }
                        buf => *buf = Buffer::Heap(vec![0; 1024]),
                    }
                }
            }
        }
    }
}

#[test]
fn chars() {
    let boundaries = "🐈°文字化けしない"
        .as_bytes()
        .iter()
        .map(|&ch| if is_utf8_char_boundary(ch) { '!' } else { '.' })
        .collect::<String>();
    assert_eq!("!...!.!..!..!..!..!..!..!..", boundaries);
}

#[test]
fn utf8_fragments() {
    let text = "▀▄ ɯopuɐɹ ⓤⓝⓘⓒⓞⓓⓔ and ascii 🐳 sʇuıodǝpoɔ ✴";
    for len in 1..9 {
        let mut out = Vec::new();
        let mut handler = |ch: &[u8]| out.extend_from_slice(ch);
        let mut t = StreamingHandlerSink::new(UTF_8, &mut handler);
        for (nth, chunk) in text.as_bytes().chunks(len).enumerate() {
            let msg = format!(
                "{len} at {nth} '{chunk:?}'; {:?}[..{}]",
                t.incomplete_bytes, t.incomplete_bytes_len
            );
            t.write_utf8_chunk(chunk, ContentType::Html).expect(&msg);
        }
        drop(t);
        assert_eq!(String::from_utf8_lossy(&out), text, "{len}");
    }
}
