//! All standard tag names contain only ASCII alpha characters
//! and digits from 1 to 6 (in numbered header tags, i.e. <h1> - <h6>).
//! Considering that tag names are case insensitive we have only
//! 26 + 6 = 32 characters. Thus, single character can be encoded in
//! 5 bits and we can fit up to 64 / 5 ≈ 12 characters in a 64-bit
//! integer. This is enough to encode all standard tag names, so
//! we can just compare integers instead of expensive string
//! comparison for tag names.
//!
//! The original idea of this tag hash-like thing belongs to Ingvar
//! Stepanyan and was implemented in lazyhtml. So, kudos to him for
//! comming up with this cool optimisation. This implementation differs
//! from the original one as it adds ability to encode digits from 1
//! to 6 which allows us to encode numbered header tags.
//!
//! In this implementation we reserve numbers from 0 to 5 for digits
//! from 1 to 6 and numbers from 6 to 31 for ASCII alphas. Otherwise,
//! if we use numbers from 0 to 25 for ASCII alphas we'll have an
//! ambiguity for repetitative `a` characters: both `a`,
//! `aaa` and even `aaaaa` will give us 0 as a hash. It's still a case
//! for digits, but considering that tag name can't start with a digit
//! we are safe here, since we'll just get first character shifted left
//! by zeroes as repetitave 1 digits get added to the hash.
use super::Tag;

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct LocalNameHash(Option<u64>);

impl LocalNameHash {
    #[inline]
    pub fn new() -> Self {
        LocalNameHash(Some(0))
    }

    #[inline]
    pub fn empty() -> Self {
        LocalNameHash::default()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    #[inline]
    pub fn update(&mut self, ch: u8) {
        if let Some(h) = self.0 {
            // NOTE: check if we still have space for yet another
            // character and if not then invalidate the hash.
            // Note, that we can't have `1` (which is encoded as 0b00000) as
            // a first character of a tag name, so it's safe to perform
            // check this way.
            self.0 = if h >> (64 - 5) == 0 {
                match ch {
                    // NOTE: apply 0x1F mask on ASCII alpha to convert it to the
                    // number from 1 to 26 (character case is controlled by one of
                    // upper bits which we eliminate with the mask). Then add
                    // 5, since numbers from 0 to 5 are reserved for digits.
                    // Aftwerards put result as 5 lower bits of the hash.
                    b'a'..=b'z' | b'A'..=b'Z' => Some((h << 5) | ((u64::from(ch) & 0x1F) + 5)),

                    // NOTE: apply 0x0F mask on ASCII digit to convert it to number
                    // from 1 to 6. Then substract 1 to make it zero-based.
                    // Afterwards, put result as lower bits of the hash.
                    b'1'..=b'6' => Some((h << 5) | ((u64::from(ch) & 0x0F) - 1)),

                    // NOTE: for any other characters hash function is not
                    // applicable, so we completely invalidate the hash.
                    _ => None,
                }
            } else {
                None
            };
        }
    }
}

impl From<&str> for LocalNameHash {
    #[inline]
    fn from(string: &str) -> Self {
        let mut hash = LocalNameHash::new();

        for ch in string.bytes() {
            hash.update(ch);
        }

        hash
    }
}

impl PartialEq<Tag> for LocalNameHash {
    #[inline]
    fn eq(&self, tag: &Tag) -> bool {
        match self.0 {
            Some(h) => *tag as u64 == h,
            None => false,
        }
    }
}

impl Eq for LocalNameHash {}
