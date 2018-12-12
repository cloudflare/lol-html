use crate::base::{Buffer, Chunk};
use crate::tokenizer::{LexUnitHandler, TagLexUnitHandler, TagPreviewHandler, Tokenizer};
use failure::{Error, ResultExt};

const BUFFER_ERROR_CONTEXT: &str = concat!(
    "This is caused by the parser encountering an extremely long ",
    "tag or a comment that is captured by the specified selector."
);

pub struct TransformStream<LH, TH, PH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
    PH: TagPreviewHandler,
{
    tokenizer: Tokenizer<LH, TH, PH>,
    buffer: Buffer,
    has_buffered_data: bool,
    finished: bool,
}

impl<LH, TH, PH> TransformStream<LH, TH, PH>
where
    LH: LexUnitHandler,
    TH: TagLexUnitHandler,
    PH: TagPreviewHandler,
{
    pub fn new(
        buffer_capacity: usize,
        lex_unit_handler: LH,
        tag_lex_unit_handler: TH,
        tag_preview_handler: PH,
    ) -> Self {
        TransformStream {
            tokenizer: Tokenizer::new(lex_unit_handler, tag_lex_unit_handler, tag_preview_handler),
            buffer: Buffer::new(buffer_capacity),
            has_buffered_data: false,
            finished: false,
        }
    }

    fn buffer_blocked_bytes(
        &mut self,
        data: &[u8],
        blocked_byte_count: usize,
    ) -> Result<(), Error> {
        if self.has_buffered_data {
            self.buffer.shrink_to_last(blocked_byte_count);
        } else {
            let blocked_bytes = &data[data.len() - blocked_byte_count..];

            self.buffer
                .init_with(blocked_bytes)
                .context(BUFFER_ERROR_CONTEXT)?;

            self.has_buffered_data = true;
        }

        trace!(@buffer self.buffer);

        Ok(())
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        assert!(!self.finished, "Attempt to call write() after end()");
        trace!(@write data);

        let chunk = if self.has_buffered_data {
            self.buffer.append(data).context(BUFFER_ERROR_CONTEXT)?;
            self.buffer.bytes()
        } else {
            data
        }
        .into();

        trace!(@chunk chunk);

        let blocked_byte_count = self.tokenizer.tokenize(&chunk)?;

        if blocked_byte_count > 0 {
            self.buffer_blocked_bytes(data, blocked_byte_count)?;
        } else {
            self.has_buffered_data = false;
        }

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), Error> {
        assert!(!self.finished, "Attempt to call end() twice");
        trace!(@end);

        self.finished = true;

        let chunk = if self.has_buffered_data {
            Chunk::last(self.buffer.bytes())
        } else {
            Chunk::last_empty()
        };

        trace!(@chunk chunk);

        self.tokenizer.tokenize(&chunk)?;

        Ok(())
    }

    #[cfg(feature = "testing_api")]
    pub fn tokenizer(&mut self) -> &mut Tokenizer<LH, TH, PH> {
        &mut self.tokenizer
    }
}
