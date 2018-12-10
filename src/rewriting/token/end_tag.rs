use crate::base::Bytes;

#[derive(Getters, Debug)]
pub struct EndTag<'i> {
    #[get = "pub"]
    name: Bytes<'i>,
}

impl<'i> EndTag<'i> {
    pub(super) fn new_parsed(name: Bytes<'i>) -> Self {
        EndTag { name }
    }
}
