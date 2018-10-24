use base::Input;

#[derive(Debug)]
pub struct Chunk<'b> {
    data: &'b [u8],
    last: bool,
}

impl<'b> Chunk<'b> {
    pub fn last() -> Self {
        Chunk {
            data: &[],
            last: true,
        }
    }
}

impl<'b> From<&'b [u8]> for Chunk<'b> {
    fn from(data: &'b [u8]) -> Self {
        Chunk { data, last: false }
    }
}

impl<'b> Input<'b> for Chunk<'b> {
    #[inline]
    fn is_last(&self) -> bool {
        self.last
    }

    #[inline]
    fn get_data(&self) -> &[u8] {
        self.data
    }
}
