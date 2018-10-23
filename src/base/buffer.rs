use base::Input;
use errors::Error;
use safemem::copy_over;
use std::ops::Deref;

#[derive(Debug)]
pub struct Buffer {
    data: Box<[u8]>,
    capacity: usize,
    watermark: usize,
    next_pos: usize,
    last: bool,
}

impl Buffer {
    pub fn new(capacity: usize) -> Self {
        Buffer {
            data: vec![0; capacity].into(),
            capacity,
            watermark: 0,
            next_pos: 0,
            last: false,
        }
    }

    pub fn mark_as_last_input(&mut self) {
        self.last = true;
    }

    pub fn append(&mut self, slice: &[u8]) -> Result<(), Error> {
        let slice_len = slice.len();

        if self.watermark + slice_len <= self.capacity {
            let new_watermark = self.watermark + slice_len;

            self.data[self.watermark..new_watermark].copy_from_slice(&slice);
            self.watermark = new_watermark;

            Ok(())
        } else {
            Err(Error::BufferCapacityExceeded)
        }
    }

    #[inline]
    pub fn clean_and_consume(&mut self, slice: &[u8]) -> Result<(), Error> {
        self.watermark = 0;

        self.append(slice)
    }

    pub fn shrink_to_last(&mut self, byte_count: usize) {
        copy_over(&mut self.data, self.watermark - byte_count, 0, byte_count);

        self.watermark = byte_count;
    }
}

impl Input for Buffer {
    #[inline]
    fn get_next_pos(&self) -> usize {
        self.next_pos
    }

    #[inline]
    fn set_next_pos(&mut self, pos: usize) {
        self.next_pos = pos;
    }

    #[inline]
    fn is_last(&self) -> bool {
        self.last
    }
}

impl Deref for Buffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        &self.data[..self.watermark]
    }
}
