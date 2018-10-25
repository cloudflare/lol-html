use errors::Error;
use safemem::copy_over;

pub struct Buffer {
    data: Box<[u8]>,
    capacity: usize,
    watermark: usize,
}

impl Buffer {
    pub fn new(capacity: usize) -> Self {
        Buffer {
            data: vec![0; capacity].into(),
            capacity,
            watermark: 0,
        }
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

    pub fn init_with(&mut self, slice: &[u8]) -> Result<(), Error> {
        self.watermark = 0;

        self.append(slice)
    }

    pub fn shrink_to_last(&mut self, byte_count: usize) {
        copy_over(&mut self.data, self.watermark - byte_count, 0, byte_count);

        self.watermark = byte_count;
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data[..self.watermark]
    }
}
