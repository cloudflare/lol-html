use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct RawStringVec(Vec<Option<String>>);

impl RawStringVec {
    pub fn push_raw(&mut self, raw: Option<&[u8]>) {
        self.push(raw.map(|b| unsafe { String::from_utf8_unchecked(b.to_vec()) }));
    }

    pub fn get_cumulative_raw_string(&self) -> String {
        self.iter().fold(String::new(), |c, s| {
            c + s.as_ref().unwrap_or(&String::new())
        })
    }
}

impl Deref for RawStringVec {
    type Target = Vec<Option<String>>;

    fn deref(&self) -> &Vec<Option<String>> {
        &self.0
    }
}

impl DerefMut for RawStringVec {
    fn deref_mut(&mut self) -> &mut Vec<Option<String>> {
        &mut self.0
    }
}
