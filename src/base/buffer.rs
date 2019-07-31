use safemem::copy_over;

#[derive(Fail, Debug, PartialEq)]
#[fail(display = "The buffer capacity ({}B) has been exceeded.", capacity)]
pub struct BufferCapacityExceededError {
    pub capacity: usize,
}

impl BufferCapacityExceededError {
    fn new(capacity: usize) -> Self {
        BufferCapacityExceededError { capacity }
    }
}

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

    pub fn append(&mut self, slice: &[u8]) -> Result<(), BufferCapacityExceededError> {
        let slice_len = slice.len();

        if self.watermark + slice_len <= self.capacity {
            let new_watermark = self.watermark + slice_len;

            self.data[self.watermark..new_watermark].copy_from_slice(&slice);
            self.watermark = new_watermark;

            Ok(())
        } else {
            Err(BufferCapacityExceededError::new(self.capacity))
        }
    }

    pub fn init_with(&mut self, slice: &[u8]) -> Result<(), BufferCapacityExceededError> {
        self.watermark = 0;

        self.append(slice)
    }

    pub fn shift(&mut self, byte_count: usize) {
        let new_watermark = self.watermark - byte_count;

        copy_over(&mut self.data, byte_count, 0, new_watermark);

        self.watermark = new_watermark;
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data[..self.watermark]
    }
}
